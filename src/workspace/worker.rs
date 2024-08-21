use crate::{
	api::{controller::ControllerWorker, Controller, Event, User},
	buffer::{self, worker::BufferWorker},
	cursor::{self, worker::CursorWorker},
	workspace::service::Services,
};

use codemp_proto::{
	auth::Token,
	common::Empty,
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

#[derive(Debug, Clone)]
#[cfg_attr(feature = "python", pyo3::pyclass)]
#[cfg_attr(feature = "js", napi)]
pub struct Workspace(Arc<WorkspaceInner>);

#[derive(Debug)]
struct WorkspaceInner {
	id: String,
	user_id: Uuid, // reference to global user id
	cursor: cursor::Controller,
	buffers: DashMap<String, buffer::Controller>,
	filetree: DashSet<String>,
	users: DashMap<Uuid, User>,
	services: Services,
	// TODO can we drop the mutex?
	events: tokio::sync::Mutex<mpsc::UnboundedReceiver<crate::api::Event>>,
}

impl Workspace {
	/// create a new buffer and perform initial fetch operations
	pub(crate) async fn try_new(
		id: String,
		user_id: Uuid,
		dest: &str,
		token: Token,
	) -> crate::Result<Self> {
		let services = Services::try_new(dest, token).await?;
		let ws_stream = services.ws().attach(Empty {}).await?.into_inner();

		let (tx, rx) = mpsc::channel(256);
		let (ev_tx, ev_rx) = mpsc::unbounded_channel();
		let cur_stream = services
			.cur()
			.attach(tokio_stream::wrappers::ReceiverStream::new(rx))
			.await?
			.into_inner();

		let worker = CursorWorker::default();
		let controller = worker.controller();
		tokio::spawn(async move {
			tracing::debug!("controller worker started");
			worker.work(tx, cur_stream).await;
			tracing::debug!("controller worker stopped");
		});

		let ws = Self(Arc::new(WorkspaceInner {
			id,
			user_id,
			cursor: controller,
			buffers: DashMap::default(),
			filetree: DashSet::default(),
			users: DashMap::default(),
			events: tokio::sync::Mutex::new(ev_rx),
			services,
		}));

		ws.fetch_users().await?;
		ws.fetch_buffers().await?;
		ws.run_actor(ws_stream, ev_tx);

		Ok(ws)
	}

	pub(crate) fn run_actor(
		&self,
		mut stream: Streaming<WorkspaceEvent>,
		tx: mpsc::UnboundedSender<crate::api::Event>,
	) {
		// TODO for buffer and cursor controller we invoke the tokio::spawn outside, but here inside..?
		let inner = self.0.clone();
		let name = self.id();
		tokio::spawn(async move {
			loop {
				match stream.message().await {
					Err(e) => break tracing::error!("workspace '{}' stream closed: {}", name, e),
					Ok(None) => break tracing::info!("leaving workspace {}", name),
					Ok(Some(WorkspaceEvent { event: None })) => {
						tracing::warn!("workspace {} received empty event", name)
					}
					Ok(Some(WorkspaceEvent { event: Some(ev) })) => {
						let update = crate::api::Event::from(&ev);
						match ev {
							// user
							WorkspaceEventInner::Join(UserJoin { user }) => {
								inner
									.users
									.insert(user.clone().into(), User { id: user.into() });
							}
							WorkspaceEventInner::Leave(UserLeave { user }) => {
								inner.users.remove(&user.into());
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
								if let Some((_name, controller)) = inner.buffers.remove(&path) {
									controller.stop();
								}
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

impl Workspace {
	/// create a new buffer in current workspace
	pub async fn create(&self, path: &str) -> crate::Result<()> {
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

	/// attach to a buffer, starting a buffer controller and returning a new reference to it
	///
	/// to interact with such buffer use [crate::api::Controller::send] or
	/// [crate::api::Controller::recv] to exchange [crate::api::TextChange]
	pub async fn attach(&self, path: &str) -> crate::Result<buffer::Controller> {
		let mut worskspace_client = self.0.services.ws();
		let request = tonic::Request::new(BufferNode {
			path: path.to_string(),
		});
		let credentials = worskspace_client.access_buffer(request).await?.into_inner();
		self.0.services.set_token(credentials.token);

		let (tx, rx) = mpsc::channel(256);
		let mut req = tonic::Request::new(tokio_stream::wrappers::ReceiverStream::new(rx));
		req.metadata_mut().insert(
			"path",
			tonic::metadata::MetadataValue::try_from(credentials.id.id)
				.expect("could not represent path as byte sequence"),
		);
		let stream = self.0.services.buf().attach(req).await?.into_inner();

		let worker = BufferWorker::new(self.0.user_id, path);
		let controller = worker.controller();
		tokio::spawn(async move {
			tracing::debug!("controller worker started");
			worker.work(tx, stream).await;
			tracing::debug!("controller worker stopped");
		});

		self.0.buffers.insert(path.to_string(), controller.clone());

		Ok(controller)
	}

	/// detach from an active buffer
	///
	/// this option will be carried in background: [buffer::worker::BufferWorker] will be stopped and dropped. there
	/// may still be some events enqueued in buffers to poll, but the [buffer::Controller] itself won't be
	/// accessible anymore from [Workspace].
	///
	/// ### returns
	/// [DetachResult::NotAttached] if buffer wasn't attached in the first place
	/// [DetachResult::Detaching] if detach was correctly requested
	/// [DetachResult::AlreadyDetached] if worker is already stopped
	pub fn detach(&self, path: &str) -> DetachResult {
		match self.0.buffers.remove(path) {
			None => DetachResult::NotAttached,
			Some((_name, controller)) => {
				if controller.stop() {
					DetachResult::Detaching
				} else {
					DetachResult::AlreadyDetached
				}
			}
		}
	}

	/// await next workspace [crate::api::Event] and return it
	pub async fn event(&self) -> crate::Result<Event> {
		self.0
			.events
			.lock()
			.await
			.recv()
			.await
			.ok_or(crate::Error::Channel { send: false })
	}

	/// fetch a list of all buffers in a workspace
	pub async fn fetch_buffers(&self) -> crate::Result<()> {
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

	/// fetch a list of all users in a workspace
	pub async fn fetch_users(&self) -> crate::Result<()> {
		let mut workspace_client = self.0.services.ws();
		let users = BTreeSet::from_iter(
			workspace_client
				.list_users(tonic::Request::new(Empty {}))
				.await?
				.into_inner()
				.users
				.into_iter()
				.map(Uuid::from),
		);

		self.0.users.clear();
		for u in users {
			self.0.users.insert(u, User { id: u });
		}

		Ok(())
	}

	/// get a list of the users attached to a specific buffer
	///
	/// TODO: discuss implementation details
	pub async fn list_buffer_users(&self, path: &str) -> crate::Result<Vec<User>> {
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

	/// delete a buffer
	pub async fn delete(&self, path: &str) -> crate::Result<()> {
		let mut workspace_client = self.0.services.ws();
		workspace_client
			.delete_buffer(tonic::Request::new(BufferNode {
				path: path.to_string(),
			}))
			.await?;

		if let Some((_name, controller)) = self.0.buffers.remove(path) {
			controller.stop();
		}

		self.0.filetree.remove(path);

		Ok(())
	}

	/// get the id of the workspace
	// #[cfg_attr(feature = "js", napi)] // https://github.com/napi-rs/napi-rs/issues/1120
	pub fn id(&self) -> String {
		self.0.id.clone()
	}

	/// return a reference to current cursor controller, if currently in a workspace
	// #[cfg_attr(feature = "js", napi)] // https://github.com/napi-rs/napi-rs/issues/1120
	pub fn cursor(&self) -> cursor::Controller {
		self.0.cursor.clone()
	}

	/// get a new reference to a buffer controller, if any is active to given path
	// #[cfg_attr(feature = "js", napi)] // https://github.com/napi-rs/napi-rs/issues/1120
	pub fn buffer_by_name(&self, path: &str) -> Option<buffer::Controller> {
		self.0.buffers.get(path).map(|x| x.clone())
	}

	/// get a list of all the currently attached to buffers
	// #[cfg_attr(feature = "js", napi)] // https://github.com/napi-rs/napi-rs/issues/1120
	pub fn buffer_list(&self) -> Vec<String> {
		self.0
			.buffers
			.iter()
			.map(|elem| elem.key().clone())
			.collect()
	}

	/// get the currently cached "filetree"
	// #[cfg_attr(feature = "js", napi)] // https://github.com/napi-rs/napi-rs/issues/1120
	pub fn filetree(&self, filter: Option<&str>) -> Vec<String> {
		self.0.filetree.iter()
			.filter(|f| filter.map_or(true, |flt| f.starts_with(flt)))
			.map(|f| f.clone())
			.collect()
	}
}

impl Drop for WorkspaceInner {
	fn drop(&mut self) {
		for entry in self.buffers.iter() {
			if !entry.value().stop() {
				tracing::warn!(
					"could not stop buffer worker {} for workspace {}",
					entry.value().name(),
					self.id
				);
			}
		}
		if !self.cursor.stop() {
			tracing::warn!("could not stop cursor worker for workspace {}", self.id);
		}
	}
}

#[cfg_attr(feature = "python", pyo3::pyclass(eq, eq_int))]
#[cfg_attr(feature = "python", derive(PartialEq))]
pub enum DetachResult {
	NotAttached,
	Detaching,
	AlreadyDetached,
}
