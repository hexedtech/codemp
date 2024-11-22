//! ### Workspace
//! A workspace represents a development environment. It contains any number of buffers and
//! tracks cursor movements across them.
//! Buffers are typically organized in a filetree-like reminiscent of POSIX filesystems.

use crate::{
	api::{
		controller::{AsyncReceiver, ControllerCallback},
		Event, User,
	},
	buffer, cursor,
	errors::{ConnectionResult, ControllerResult, RemoteResult},
	ext::{IgnorableError, InternallyMutable},
	network::Services,
};

use codemp_proto::{
	common::{Empty, Token},
	files::BufferNode,
	workspace::{
		workspace_event::{
			Event as WorkspaceEventInner, FileCreate, FileDelete, FileRename, UserJoin, UserLeave,
		},
		WorkspaceEvent,
	},
};

use dashmap::{DashMap, DashSet};
use std::sync::{Arc, Weak};
use tokio::sync::{mpsc::{self, error::TryRecvError}, oneshot, watch};
use tonic::Streaming;
use uuid::Uuid;

#[cfg(feature = "js")]
use napi_derive::napi;

/// A currently active shared development environment
///
/// Workspaces encapsulate a working environment: cursor positions, filetree, user list
/// and more. Each holds a [`cursor::Controller`] and a map of [`buffer::Controller`]s.
/// Using a workspace handle, it's possible to receive events (user join/leave, filetree updates)
/// and create/delete/attach to new buffers.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "py", pyo3::pyclass)]
#[cfg_attr(feature = "js", napi)]
pub struct Workspace(Arc<WorkspaceInner>);

#[derive(Debug)]
struct WorkspaceInner {
	name: String,
	current_user: Arc<User>,
	cursor: cursor::Controller,
	buffers: DashMap<String, buffer::Controller>,
	services: Services,
	filetree: DashSet<String>,
	users: Arc<DashMap<Uuid, User>>,
	events: tokio::sync::Mutex<mpsc::UnboundedReceiver<crate::api::Event>>,
	callback: watch::Sender<Option<ControllerCallback<Workspace>>>,
	poll_tx: mpsc::UnboundedSender<oneshot::Sender<()>>,
}

impl AsyncReceiver<Event> for Workspace {
	async fn try_recv(&self) -> ControllerResult<Option<Event>> {
		match self.0.events.lock().await.try_recv() {
			Ok(x) => Ok(Some(x)),
			Err(TryRecvError::Empty) => Ok(None),
			Err(TryRecvError::Disconnected) => Err(crate::errors::ControllerError::Stopped),
		}
	}

	async fn poll(&self) -> ControllerResult<()> {
		let (tx, rx) = oneshot::channel();
		self.0.poll_tx.send(tx)?;
		Ok(rx.await?)
	}

	fn clear_callback(&self) {
		self.0.callback.send_replace(None);
	}

	fn callback(&self, cb: impl Into<ControllerCallback<Self>>) {
		self.0.callback.send_replace(Some(cb.into()));
	}
}

impl Workspace {
	pub(crate) async fn connect(
		name: String,
		user: Arc<User>,
		config: crate::api::Config,
		token: Token,
		claims: tokio::sync::watch::Receiver<codemp_proto::common::Token>,
	) -> ConnectionResult<Self> {
		let workspace_claim = InternallyMutable::new(token);
		let services =
			Services::try_new(&config.endpoint(), claims, workspace_claim.channel()).await?;
		let ws_stream = services.ws().attach(Empty {}).await?.into_inner();

		let (tx, rx) = mpsc::channel(128);
		let (ev_tx, ev_rx) = mpsc::unbounded_channel();
		let (poll_tx, poll_rx) = mpsc::unbounded_channel();
		let (cb_tx, cb_rx) = watch::channel(None);
		let cur_stream = services
			.cur()
			.attach(tokio_stream::wrappers::ReceiverStream::new(rx))
			.await?
			.into_inner();

		let users = Arc::new(DashMap::default());
		let controller = cursor::Controller::spawn(users.clone(), tx, cur_stream, &name);

		let ws = Self(Arc::new(WorkspaceInner {
			name: name.clone(),
			current_user: user,
			cursor: controller,
			buffers: DashMap::default(),
			filetree: DashSet::default(),
			users,
			events: tokio::sync::Mutex::new(ev_rx),
			services,
			callback: cb_tx,
			poll_tx,
		}));

		let weak = Arc::downgrade(&ws.0);

		let worker = WorkspaceWorker {
			callback: cb_rx,
			pollers: Vec::new(),
			poll_rx,
			events: ev_tx,
		};

		let _t = tokio::spawn(async move {
			worker.work(name, ws_stream, weak).await;
		});

		ws.fetch_users().await?;
		ws.fetch_buffers().await?;

		Ok(ws)
	}

	/// drop arc, return true if was last
	pub(crate) fn consume(self) -> bool {
		Arc::into_inner(self.0).is_some()
	}

	/// Create a new buffer in the current workspace.
	pub async fn create_buffer(&self, path: &str) -> RemoteResult<()> {
		let mut workspace_client = self.0.services.ws();
		workspace_client
			.create_buffer(tonic::Request::new(BufferNode {
				path: path.to_string(),
			}))
			.await?;

		// add to filetree
		self.0.filetree.insert(path.to_string());

		// fetch buffers
		self.fetch_buffers().await?;

		Ok(())
	}

	/// Attach to a buffer and return a handle to it.
	pub async fn attach_buffer(&self, path: &str) -> ConnectionResult<buffer::Controller> {
		let mut worskspace_client = self.0.services.ws();
		let request = tonic::Request::new(BufferNode {
			path: path.to_string(),
		});
		let credentials = worskspace_client.access_buffer(request).await?.into_inner();

		let (tx, rx) = mpsc::channel(256);
		let mut req = tonic::Request::new(tokio_stream::wrappers::ReceiverStream::new(rx));
		req.metadata_mut().insert(
			"buffer",
			tonic::metadata::MetadataValue::try_from(credentials.token).map_err(|e| {
				tonic::Status::internal(format!("failed representing token to string: {e}"))
			})?,
		);
		let stream = self.0.services.buf().attach(req).await?.into_inner();

		let controller = buffer::Controller::spawn(self.0.current_user.id, path, tx, stream, &self.0.name);
		self.0.buffers.insert(path.to_string(), controller.clone());

		Ok(controller)
	}

	/// Detach from an active buffer.
	///
	/// This will stop and drop its [`buffer::Controller`].
	///
	/// Returns `true` if it was connectly dropped or wasn't present, `false` if it was dropped but
	/// wasn't the last existing reference to it. If this method returns `false` it means you have
	/// a dangling reference somewhere. It may just be waiting for garbage collection, but as long
	/// as it exists, it will prevent the controller from being completely dropped.
	#[allow(clippy::redundant_pattern_matching)] // all cases are clearer this way
	pub fn detach_buffer(&self, path: &str) -> bool {
		match self.0.buffers.remove(path) {
			None => true, // noop: we werent attached in the first place
			Some((_name, controller)) => match Arc::into_inner(controller.0) {
				None => false,   // dangling ref! we can't drop this
				Some(_) => true, // dropping it now
			},
		}
	}

	/// Re-fetch the list of available buffers in the workspace.
	pub async fn fetch_buffers(&self) -> RemoteResult<Vec<String>> {
		let mut workspace_client = self.0.services.ws();
		let resp = workspace_client
			.list_buffers(tonic::Request::new(Empty {}))
			.await?
			.into_inner();

		let mut out = Vec::new();

		self.0.filetree.clear();
		for b in resp.buffers {
			self.0.filetree.insert(b.path.clone());
			out.push(b.path);
		}

		Ok(out)
	}

	/// Re-fetch the list of all users in the workspace.
	pub async fn fetch_users(&self) -> RemoteResult<Vec<User>> {
		let mut workspace_client = self.0.services.ws();
		let users = workspace_client
			.list_users(tonic::Request::new(Empty {}))
			.await?
			.into_inner()
			.users
			.into_iter()
			.map(User::from);

		let mut result = Vec::new();

		self.0.users.clear();
		for u in users {
			self.0.users.insert(u.id, u.clone());
			result.push(u);
		}

		Ok(result)
	}

	/// Fetch a list of the [User]s attached to a specific buffer.
	pub async fn fetch_buffer_users(&self, path: &str) -> RemoteResult<Vec<User>> {
		let mut workspace_client = self.0.services.ws();
		let buffer_users = workspace_client
			.list_buffer_users(tonic::Request::new(BufferNode {
				path: path.to_string(),
			}))
			.await?
			.into_inner()
			.users
			.into_iter()
			.map(|id| id.into())
			.collect();

		Ok(buffer_users)
	}

	/// Delete a buffer.
	pub async fn delete_buffer(&self, path: &str) -> RemoteResult<()> {
		self.detach_buffer(path); // just in case

		let mut workspace_client = self.0.services.ws();
		workspace_client
			.delete_buffer(tonic::Request::new(BufferNode {
				path: path.to_string(),
			}))
			.await?;

		self.0.filetree.remove(path);

		Ok(())
	}

	/// Get the workspace unique id.
	// #[cfg_attr(feature = "js", napi)] // https://github.com/napi-rs/napi-rs/issues/1120
	pub fn id(&self) -> String {
		self.0.name.clone()
	}

	/// Return a handle to the [`cursor::Controller`].
	// #[cfg_attr(feature = "js", napi)] // https://github.com/napi-rs/napi-rs/issues/1120
	pub fn cursor(&self) -> cursor::Controller {
		self.0.cursor.clone()
	}

	/// Return a handle to the [buffer::Controller] with the given path, if present.
	// #[cfg_attr(feature = "js", napi)] // https://github.com/napi-rs/napi-rs/issues/1120
	pub fn get_buffer(&self, path: &str) -> Option<buffer::Controller> {
		self.0.buffers.get(path).map(|x| x.clone())
	}

	/// Get a list of all the currently attached buffers.
	// #[cfg_attr(feature = "js", napi)] // https://github.com/napi-rs/napi-rs/issues/1120
	pub fn active_buffers(&self) -> Vec<String> {
		self.0
			.buffers
			.iter()
			.map(|elem| elem.key().clone())
			.collect()
	}

	/// Get all names of users currently in this workspace
	pub fn user_list(&self) -> Vec<User> {
		self.0
			.users
			.iter()
			.map(|elem| elem.value().clone())
			.collect()
	}

	/// Get the filetree as it is currently cached.
	/// A filter may be applied, and it may be strict (equality check) or not (starts_with check).
	// #[cfg_attr(feature = "js", napi)] // https://github.com/napi-rs/napi-rs/issues/1120
	pub fn search_buffers(&self, filter: Option<&str>) -> Vec<String> {
		let mut tree = self
			.0
			.filetree
			.iter()
			.filter(|f| filter.map_or(true, |flt| f.starts_with(flt)))
			.map(|f| f.clone())
			.collect::<Vec<String>>();
		tree.sort();
		tree
	}
}

struct WorkspaceWorker {
	callback: watch::Receiver<Option<ControllerCallback<Workspace>>>,
	pollers: Vec<oneshot::Sender<()>>,
	poll_rx: mpsc::UnboundedReceiver<oneshot::Sender<()>>,
	events: mpsc::UnboundedSender<crate::api::Event>,
}

impl WorkspaceWorker {
	pub(crate) async fn work(mut self, name: String, mut stream: Streaming<WorkspaceEvent>, weak: Weak<WorkspaceInner>) {
		tracing::debug!("workspace worker starting");
		loop {
			tokio::select! {
				res = self.poll_rx.recv() => match res {
				None => break tracing::debug!("pollers channel closed: workspace has been dropped"),
					Some(x) => self.pollers.push(x),
				},

				res = stream.message() => match res {
					Err(e) => break tracing::error!("workspace '{}' stream closed: {}", name, e),
					Ok(None) => break tracing::info!("leaving workspace {}", name),
					Ok(Some(WorkspaceEvent { event: None })) => {
						tracing::warn!("workspace {} received empty event", name)
					}
					Ok(Some(WorkspaceEvent { event: Some(ev) })) => {
						let Some(inner) = weak.upgrade() else { break };
						let update = crate::api::Event::from(&ev);
						match ev {
							// user
							WorkspaceEventInner::Join(UserJoin { user }) => {
								inner.users.insert(user.id.uuid(), user.into());
							}
							WorkspaceEventInner::Leave(UserLeave { user }) => {
								inner.users.remove(&user.id.uuid());
							}
							// buffer
							WorkspaceEventInner::Create(FileCreate { path }) => {
								inner.filetree.insert(path);
							}
							WorkspaceEventInner::Rename(FileRename { before, after }) => {
								inner.filetree.remove(&before);
								inner.filetree.insert(after);
							}
							WorkspaceEventInner::Delete(FileDelete { path }) => {
								inner.filetree.remove(&path);
								let _ = inner.buffers.remove(&path);
							}
						}
						if self.events.send(update).is_err() {
							tracing::warn!("no active controller to receive workspace event");
						}
						self.pollers.drain(..).for_each(|x| {
							x.send(()).unwrap_or_warn("poller dropped before completion");
						});
						if let Some(cb) = self.callback.borrow().as_ref() {
							if let Some(ws) = weak.upgrade() {
								cb.call(Workspace(ws));
							} else {
								break tracing::debug!("workspace worker clean exit");
							}
						}
					}
				},
			}
		}
		tracing::debug!("workspace worker stopping");
	}
}
