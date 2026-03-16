//! ### Workspace
//! A workspace represents a development environment. It contains any number of buffers and
//! tracks cursor movements across them.
//! Buffers are typically organized in a filetree-like reminiscent of POSIX filesystems.

use crate::{
	api::{
		Event, UserInfo,
		controller::{AsyncReceiver, ControllerCallback},
	},
	buffer, cursor,
	errors::{ConnectionResult, ControllerResult, RemoteResult},
	ext::IgnorableError,
	network::Services,
};

use codemp_proto::{
	common::Empty,
	files::{BufferNode, BufferPath},
	workspace::{
		WorkspaceEvent,
		workspace_event::{
			Event as WorkspaceEventInner, FileCreate, FileDelete, FileRename, UserJoinBuffer,
			UserJoinWorkspace, UserLeaveBuffer, UserLeaveWorkspace,
		},
	},
};

use dashmap::DashMap;
use std::sync::{Arc, Weak};
use tokio::sync::{
	mpsc::{self, error::TryRecvError},
	oneshot, watch,
};
use tonic::Streaming;

#[cfg(feature = "js")]
use napi_derive::napi;

/// A currently active shared development environment
///
/// Workspaces encapsulate a working environment: cursor positions, filetree, user list
/// and more. Each holds a [`cursor::Controller`] and a map of [`buffer::Controller`]s.
/// Using a workspace handle, it's possible to receive events (user join/leave, filetree updates)
/// and create/delete/attach to new buffers.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "py", pyo3::pyclass(from_py_object))]
#[cfg_attr(feature = "js", napi)]
pub struct Workspace(pub(crate) Arc<WorkspaceInner>);

#[derive(Debug)]
pub(crate) struct WorkspaceInner {
	id: crate::api::WorkspaceIdentifier,
	current_user: Arc<UserInfo>,
	cursor: cursor::Controller,
	buffers: DashMap<String, buffer::Controller>,
	services: Services,
	filetree: DashMap<String, crate::api::BufferNode>,
	buffer_users: DashMap<String, Vec<String>>,
	users: Arc<DashMap<String, UserInfo>>,
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
	#[tracing::instrument(skip(id, user, workspace_claim, user_claim), fields(ws = %id))]
	pub(crate) async fn connect(
		id: crate::api::WorkspaceIdentifier,
		user: Arc<UserInfo>,
		config: crate::api::Config,
		workspace_claim: tokio::sync::watch::Receiver<codemp_proto::common::Token>,
		user_claim: tokio::sync::watch::Receiver<codemp_proto::common::Token>,
	) -> ConnectionResult<Self> {
		let services = Services::try_new(&config.endpoint(), user_claim, workspace_claim).await?;
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
		let controller = cursor::Controller::spawn(
			users.clone(),
			tx,
			cur_stream,
			id.clone(),
			services.cur().clone(),
		);

		let ws = Self(Arc::new(WorkspaceInner {
			id: id.clone(),
			current_user: user,
			cursor: controller,
			buffers: DashMap::default(),
			filetree: DashMap::default(),
			buffer_users: DashMap::default(),
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
			worker.work(id, ws_stream, weak).await;
		});

		ws.fetch_users().await?;
		ws.fetch_buffers().await?;

		for buffer_ref in ws.0.buffers.iter() {
			ws.fetch_buffer_users(buffer_ref.key().clone()).await?;
		}

		Ok(ws)
	}

	pub(crate) fn services(&self) -> &Services {
		&self.0.services
	}

	/// drop arc, return true if was last
	pub(crate) fn consume(self) -> bool {
		Arc::into_inner(self.0).is_some()
	}

	/// Create a new buffer in the current workspace.
	pub async fn create_buffer(&self, path: impl ToString, ephemeral: bool) -> RemoteResult<()> {
		let mut workspace_client = self.0.services.ws();
		workspace_client
			.create_buffer(tonic::Request::new(BufferNode {
				path: path.to_string().into(),
				ephemeral,
			}))
			.await?;

		// add to filetree, not really necessary as we will get an event for it
		self.0.filetree.insert(
			path.to_string(),
			crate::api::BufferNode {
				path: path.to_string(),
				ephemeral,
			},
		);

		Ok(())
	}

	/// Pin an ephemeral buffer, making it permanent.
	pub async fn pin_buffer(&self, path: impl AsRef<str>) -> RemoteResult<()> {
		self.0
			.services
			.ws()
			.clone()
			.pin_buffer(BufferPath::from(path.as_ref()))
			.await?;
		Ok(())
	}

	/// Unpins a permanen buffer, making it ephemeral.
	pub async fn un_pin_buffer(&self, path: impl AsRef<str>) -> RemoteResult<()> {
		self.0
			.services
			.ws()
			.clone()
			.un_pin_buffer(BufferPath::from(path.as_ref()))
			.await?;
		Ok(())
	}

	/// Attach to a buffer and return a handle to it.
	#[tracing::instrument(skip(self, path), fields(path = path.to_string()))]
	pub async fn attach_buffer(&self, path: impl ToString) -> ConnectionResult<buffer::Controller> {
		let path = path.to_string();
		let mut workspace_client = self.0.services.ws();
		let mut buffer_client = self.0.services.buf();
		let credentials = workspace_client
			.get_buffer_token(BufferPath::from(&path))
			.await?
			.into_inner();

		let (tx, rx) = mpsc::channel(256);
		let mut req = tonic::Request::new(tokio_stream::wrappers::ReceiverStream::new(rx));
		req.metadata_mut()
			.insert("buffer", crate::ext::token_to_metadata(credentials)?);
		let stream = buffer_client.attach(req).await?.into_inner();

		let controller = buffer::Controller::spawn(
			self.0.current_user.name.clone(),
			path.clone(),
			tx,
			stream,
			self.0.id.clone(),
		);

		self.0.buffers.insert(path.clone(), controller.clone());

		let _path = path.clone();
		let weak = Arc::downgrade(&controller.0);
		tokio::spawn(async move {
			let fut = async move {
				loop {
					// TODO either configurable token refresh time or calculate depending on token lifetime
					tokio::time::sleep(std::time::Duration::from_secs(20)).await;
					if weak.upgrade().is_none() {
						break;
					};
					let new_credentials = workspace_client
						.get_buffer_token(BufferPath::from(&_path))
						.await?
						.into_inner();
					let mut request = tonic::Request::new(Empty {});
					request
						.metadata_mut()
						.insert("buffer", crate::ext::token_to_metadata(new_credentials)?);
					buffer_client.keep_alive(request).await?;
				}
				Ok::<(), tonic::Status>(())
			};

			if let Err(e) = fut.await {
				tracing::error!("error in keepalive task for buffer {path}: {e}");
			}
		});

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
	pub fn detach_buffer(&self, path: impl AsRef<str>) -> bool {
		match self.0.buffers.remove(path.as_ref()) {
			None => true, // noop: we werent attached in the first place
			Some((_name, controller)) => match Arc::into_inner(controller.0) {
				None => false,   // dangling ref! we can't drop this
				Some(_) => true, // dropping it now
			},
		}
	}

	/// Re-fetch the list of available buffers in the workspace.
	pub async fn fetch_buffers(&self) -> RemoteResult<()> {
		let mut workspace_client = self.0.services.ws();
		let resp = workspace_client.fetch_buffers(Empty {}).await?.into_inner();

		self.0.filetree.clear();
		for b in resp.buffers {
			self.0
				.filetree
				.insert(b.path.clone().into(), crate::api::BufferNode::from(b));
		}

		Ok(())
	}

	/// Re-fetch the list of all users in the workspace.
	pub async fn fetch_users(&self) -> RemoteResult<()> {
		let mut workspace_client = self.services().ws();
		let resp = workspace_client.fetch_users(Empty {}).await?.into_inner();

		self.0.users.clear();
		for user_name in resp.users {
			// TODO need to fetch whole user profiles here maybe?
			self.0
				.users
				.insert(user_name.clone(), UserInfo::default_for(user_name));
		}

		Ok(())
	}

	/// Fetch a list of the [User]s attached to a specific buffer.
	pub async fn fetch_buffer_users(&self, path: impl ToString) -> RemoteResult<()> {
		let path = path.to_string();
		let resp = self
			.services()
			.ws()
			.fetch_buffer_users(BufferPath::from(&path))
			.await?
			.into_inner();

		self.0.buffer_users.insert(path, resp.users);

		Ok(())
	}

	/// Delete a buffer.
	pub async fn delete_buffer(&self, path: impl AsRef<str>) -> RemoteResult<()> {
		self.detach_buffer(path.as_ref()); // just in case

		let mut workspace_client = self.0.services.ws();
		workspace_client
			.delete_buffer(BufferPath::from(path.as_ref()))
			.await?;

		self.0.filetree.remove(path.as_ref());

		Ok(())
	}

	/// Get the workspace unique id.
	// #[cfg_attr(feature = "js", napi)] // https://github.com/napi-rs/napi-rs/issues/1120
	pub fn id(&self) -> &crate::api::WorkspaceIdentifier {
		&self.0.id
	}

	/// Return a handle to the [`cursor::Controller`].
	// #[cfg_attr(feature = "js", napi)] // https://github.com/napi-rs/napi-rs/issues/1120
	pub fn cursor(&self) -> cursor::Controller {
		self.0.cursor.clone()
	}

	/// Return a handle to the [buffer::Controller] with the given path, if present.
	// #[cfg_attr(feature = "js", napi)] // https://github.com/napi-rs/napi-rs/issues/1120
	pub fn get_buffer(&self, path: impl AsRef<str>) -> Option<buffer::Controller> {
		self.0.buffers.get(path.as_ref()).map(|x| x.clone())
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

	/// Get all users currently in this workspace
	pub fn user_list(&self) -> Vec<UserInfo> {
		self.0
			.users
			.iter()
			.map(|elem| elem.value().clone())
			.collect()
	}

	/// Get all users currently attached to specified buffer
	pub fn buffer_user_list(&self, path: impl AsRef<str>) -> Vec<UserInfo> {
		let mut out = Vec::new();
		if let Some(buf_ref) = self.0.buffer_users.get(path.as_ref()) {
			for uid in buf_ref.value() {
				if let Some(user_ref) = self.0.users.get(uid) {
					out.push(user_ref.value().clone());
				}
			}
		}
		out
	}

	/// Get the filetree as it is currently cached.
	/// A filter may be applied, and it works as a "starts_with" check.
	// #[cfg_attr(feature = "js", napi)] // https://github.com/napi-rs/napi-rs/issues/1120
	pub fn search_buffers(&self, filter: Option<&str>) -> Vec<String> {
		let mut tree = self
			.0
			.filetree
			.iter()
			.filter(|f| filter.is_none_or(|flt| f.key().starts_with(flt)))
			.map(|f| f.key().clone())
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
	#[tracing::instrument(skip(self, stream, weak))]
	pub(crate) async fn work(
		mut self,
		ws: crate::api::WorkspaceIdentifier,
		mut stream: Streaming<WorkspaceEvent>,
		weak: Weak<WorkspaceInner>,
	) {
		tracing::debug!("workspace worker starting");
		loop {
			tokio::select! {
				res = self.poll_rx.recv() => match res {
				None => break tracing::debug!("pollers channel closed: workspace has been dropped"),
					Some(x) => self.pollers.push(x),
				},

				res = stream.message() => match res {
					Err(e) => break tracing::error!("workspace '{ws}' stream closed: {e}"),
					Ok(None) => break tracing::info!("leaving workspace {ws}"),
					Ok(Some(WorkspaceEvent { event: None })) => {
						tracing::warn!("workspace {ws} received empty event")
					}
					Ok(Some(WorkspaceEvent { event: Some(ev) })) => {
						let Some(inner) = weak.upgrade() else {
							break tracing::debug!("workspace worker clean exit");
						};
						tracing::debug!("received workspace event: {ev:?}");
						let update = crate::api::Event::from(&ev);
						match ev {
							// user
							WorkspaceEventInner::WorkspaceJoin(UserJoinWorkspace { user }) => {
								inner.users.insert(user.clone(), UserInfo::default_for(user));
							}
							WorkspaceEventInner::WorkspaceLeave(UserLeaveWorkspace { user }) => {
								inner.users.remove(&user);
							}
							WorkspaceEventInner::BufferJoin(UserJoinBuffer { user, buffer }) => {
								match inner.buffer_users.get_mut(&buffer) {
									Some(mut buf_users_ref) => buf_users_ref.push(user),
									None => { inner.buffer_users.insert(buffer, vec![user]); },
								}
							},
							WorkspaceEventInner::BufferLeave(UserLeaveBuffer { user, buffer }) => {
								match inner.buffer_users.get_mut(&buffer) {
									Some(mut buf_users_ref) => buf_users_ref.retain(|x| *x != user),
									None => tracing::warn!("received UserLeaveBuffer event for an unknown buffer"),
								}
							},
							// buffer
							WorkspaceEventInner::Create(FileCreate { path, ephemeral }) => {
								inner.buffer_users.insert(path.clone(), Vec::new());
								inner.filetree.insert(path.clone(), crate::api::BufferNode { path, ephemeral });
							}
							WorkspaceEventInner::Rename(FileRename { before, after }) => {
								if let Some((_path, controller)) = inner.buffers.remove(&before) {
									inner.buffers.insert(after.clone(), controller);
								}
								if let Some((_path, node)) = inner.filetree.remove(&before) {
									inner.filetree.insert(after.clone(), node);
								}
								if let Some((_path, users)) = inner.buffer_users.remove(&before) {
									inner.buffer_users.insert(after, users);
								}
							}
							WorkspaceEventInner::Delete(FileDelete { path }) => {
								inner.filetree.remove(&path);
								inner.buffer_users.remove(&path);
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
								break tracing::debug!("workspace worker clean (late) exit");
							}
						}
					}
				},
			}
		}
		tracing::debug!("workspace worker stopping");
	}
}
