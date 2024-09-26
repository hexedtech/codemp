//! ### Workspace
//! A workspace represents a development environment. It contains any number of buffers and
//! tracks cursor movements across them.
//! Buffers are typically organized in a filetree-like reminiscent of POSIX filesystems.

use crate::{
	api::{Event, User},
	buffer,	cursor,
	errors::{ConnectionResult, ControllerResult, RemoteResult},
	ext::InternallyMutable,
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
use std::{collections::BTreeSet, sync::Arc};
use tokio::sync::mpsc;
use tonic::Streaming;
use uuid::Uuid;

#[cfg(feature = "js")]
use napi_derive::napi;

/// A currently active shared development environment
///
/// Workspaces encapsulate a working environment: cursor positions, filetree, user list
/// and more. Each holds a [cursor::Controller] and a map of [buffer::Controller]s.
/// Using a workspace handle, it's possible to receive events (user join/leave, filetree updates)
/// and create/delete/attach to new buffers.
#[derive(Debug, Clone)]
#[cfg_attr(any(feature = "py", feature = "py-noabi"), pyo3::pyclass)]
#[cfg_attr(feature = "js", napi)]
pub struct Workspace(Arc<WorkspaceInner>);

#[derive(Debug)]
struct WorkspaceInner {
	name: String,
	user: User, // TODO back-reference to global user id... needed for buffer controllers
	cursor: cursor::Controller,
	buffers: DashMap<String, buffer::Controller>,
	services: Services,
	// TODO these two are Arced so that the inner worker can hold them without holding the
	//      WorkspaceInner itself, otherwise its impossible to drop Workspace
	filetree: DashSet<String>,
	users: Arc<DashMap<Uuid, User>>,
	// TODO can we drop the mutex?
	events: tokio::sync::Mutex<mpsc::UnboundedReceiver<crate::api::Event>>,
}

impl Workspace {
	pub(crate) async fn connect(
		name: String,
		user: User,
		config: crate::api::Config,
		token: Token,
		claims: tokio::sync::watch::Receiver<codemp_proto::common::Token>, // TODO ughh receiving this
	) -> ConnectionResult<Self> {
		let workspace_claim = InternallyMutable::new(token);
		let services =
			Services::try_new(&config.endpoint(), claims, workspace_claim.channel()).await?;
		let ws_stream = services.ws().attach(Empty {}).await?.into_inner();

		let (tx, rx) = mpsc::channel(128);
		let (ev_tx, ev_rx) = mpsc::unbounded_channel();
		let cur_stream = services
			.cur()
			.attach(tokio_stream::wrappers::ReceiverStream::new(rx))
			.await?
			.into_inner();

		let users = Arc::new(DashMap::default());

		let controller = cursor::Controller::spawn(users.clone(), tx, cur_stream);

		let ws = Self(Arc::new(WorkspaceInner {
			name,
			user,
			cursor: controller,
			buffers: DashMap::default(),
			filetree: DashSet::default(),
			users,
			events: tokio::sync::Mutex::new(ev_rx),
			services,
		}));

		ws.fetch_users().await?;
		ws.fetch_buffers().await?;
		ws.run_actor(ws_stream, ev_tx);

		Ok(ws)
	}

	/// Create a new buffer in the current workspace.
	pub async fn create(&self, path: &str) -> RemoteResult<()> {
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
	pub async fn attach(&self, path: &str) -> ConnectionResult<buffer::Controller> {
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

		let controller = buffer::Controller::spawn(self.0.user.id, path, tx, stream);
		self.0.buffers.insert(path.to_string(), controller.clone());

		Ok(controller)
	}

	/// Detach from an active buffer.
	///
	/// This will stop and drop its [`buffer::Controller`].
	///
	/// Returns `true` if connectly dropped or wasn't present, `false` if dropped but wasn't last ref
	///
	/// If this method returns `false` you have a dangling ref, maybe just waiting for garbage
	/// collection or maybe preventing the controller from being dropped completely
	#[allow(clippy::redundant_pattern_matching)] // all cases are clearer this way
	pub fn detach(&self, path: &str) -> bool {
		match self.0.buffers.remove(path) {
			None => true, // noop: we werent attached in the first place
			Some((_name, controller)) => match Arc::into_inner(controller.0) {
				None => false, // dangling ref! we can't drop this
				Some(_) => true, // dropping it now
			}
		}
	}

	/// Await next workspace [Event] and return it when it arrives.
	// TODO this method is weird and ugly, can we make it more standard?
	pub async fn event(&self) -> ControllerResult<Event> {
		self.0
			.events
			.lock()
			.await
			.recv()
			.await
			.ok_or(crate::errors::ControllerError::Unfulfilled)
	}

	/// Re-fetch the list of available buffers in the workspace.
	pub async fn fetch_buffers(&self) -> RemoteResult<()> {
		let mut workspace_client = self.0.services.ws();
		let buffers = workspace_client
			.list_buffers(tonic::Request::new(Empty {}))
			.await?
			.into_inner()
			.buffers;

		self.0.filetree.clear();
		for b in buffers {
			self.0.filetree.insert(b.path);
		}

		Ok(())
	}

	/// Re-fetch the list of all users in the workspace.
	pub async fn fetch_users(&self) -> RemoteResult<()> {
		let mut workspace_client = self.0.services.ws();
		let users = BTreeSet::from_iter(
			workspace_client
				.list_users(tonic::Request::new(Empty {}))
				.await?
				.into_inner()
				.users
				.into_iter()
				.map(User::from),
		);

		self.0.users.clear();
		for u in users {
			self.0.users.insert(u.id, u);
		}

		Ok(())
	}

	/// Get a list of the [User]s attached to a specific buffer.
	pub async fn list_buffer_users(&self, path: &str) -> RemoteResult<Vec<User>> {
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
	pub async fn delete(&self, path: &str) -> RemoteResult<()> {
		self.detach(path); // just in case

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
	pub fn buffer_by_name(&self, path: &str) -> Option<buffer::Controller> {
		self.0.buffers.get(path).map(|x| x.clone())
	}

	/// Get a list of all the currently attached buffers.
	// #[cfg_attr(feature = "js", napi)] // https://github.com/napi-rs/napi-rs/issues/1120
	pub fn buffer_list(&self) -> Vec<String> {
		self.0
			.buffers
			.iter()
			.map(|elem| elem.key().clone())
			.collect()
	}

	/// Get all names of users currently in this workspace
	pub fn user_list(&self) -> Vec<String> {
		self.0
			.users
			.iter()
			.map(|elem| elem.value().name.clone())
			.collect()
	}

	/// Get the filetree as it is currently cached.
	/// A filter may be applied, and it may be strict (equality check) or not (starts_with check).
	// #[cfg_attr(feature = "js", napi)] // https://github.com/napi-rs/napi-rs/issues/1120
	pub fn filetree(&self, filter: Option<&str>, strict: bool) -> Vec<String> {
		self.0
			.filetree
			.iter()
			.filter(|f| {
				filter.map_or(true, |flt| {
					if strict {
						f.as_str() == flt
					} else {
						f.starts_with(flt)
					}
				})
			})
			.map(|f| f.clone())
			.collect()
	}

	pub(crate) fn run_actor(
		&self,
		mut stream: Streaming<WorkspaceEvent>,
		tx: mpsc::UnboundedSender<crate::api::Event>,
	) {
		// TODO for buffer and cursor controller we invoke the tokio::spawn outside, but here inside..?
		let weak = Arc::downgrade(&self.0);
		let name = self.id();
		tokio::spawn(async move {
			loop {
				// TODO can we stop responsively rather than poll for Arc being dropped?
				if weak.upgrade().is_none() { break };
				let Some(res) = tokio::select!(
					x = stream.message() => Some(x),
					_ = tokio::time::sleep(std::time::Duration::from_secs(5)) => None,
				) else { continue };
				match res {
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
						if tx.send(update).is_err() {
							tracing::warn!("no active controller to receive workspace event");
						}
					}
				}
			}
		});
	}
}
