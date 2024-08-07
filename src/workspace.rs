use crate::{
	api::{Controller, controller::ControllerWorker},
	buffer::{self, worker::BufferWorker},
	client::Services,
	cursor,
};
use codemp_proto::{
	auth::Token,
	common::{Empty, Identity},
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

//TODO may contain more info in the future
#[derive(Debug, Clone)]
pub struct UserInfo {
	pub uuid: Uuid,
}

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
	users: DashMap<Uuid, UserInfo>,
	token: Arc<tokio::sync::watch::Sender<Token>>, // shared
	services: Arc<Services>, // shared
}

impl Workspace {
	/// create a new buffer and perform initial fetch operations
	pub(crate) fn new(
		id: String,
		user_id: Uuid,
		cursor: cursor::Controller,
		token: Arc<tokio::sync::watch::Sender<Token>>,
		services: Arc<Services>,
	) -> Self {
		Self(Arc::new(WorkspaceInner {
			id,
			user_id,
			token,
			cursor,
			buffers: DashMap::default(),
			filetree: DashSet::default(),
			users: DashMap::default(),
			services,
		}))
	}

	pub(crate) fn run_actor(&self, mut stream: Streaming<WorkspaceEvent>) {
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
						match ev {
							WorkspaceEventInner::Join(UserJoin { user }) => {
								inner.users.insert(user.clone().into(), UserInfo { uuid: user.into() });
							}
							WorkspaceEventInner::Leave(UserLeave { user }) => {
								inner.users.remove(&user.into());
							}
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
					},
				}
			}
		});
	}

	/// create a new buffer in current workspace
	pub async fn create(&self, path: &str) -> crate::Result<()> {
		let mut workspace_client = self.0.services.workspace.clone();
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
		let mut worskspace_client = self.0.services.workspace.clone();
		let request = tonic::Request::new(BufferNode {
			path: path.to_string(),
		});
		let credentials = worskspace_client.access_buffer(request).await?.into_inner();
		self.0.token.send(credentials.token)?;

		let (tx, rx) = mpsc::channel(256);
		let mut req = tonic::Request::new(tokio_stream::wrappers::ReceiverStream::new(rx));
		req.metadata_mut().insert(
			"path",
			tonic::metadata::MetadataValue::try_from(credentials.id.id)
				.expect("could not represent path as byte sequence"),
		);
		let stream = self
			.0
			.services
			.buffer
			.clone()
			.attach(req)
			.await?
			.into_inner();

		let worker = BufferWorker::new(self.0.user_id, path);
		let controller = worker.subscribe();
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
			Some((_name, controller)) => if controller.stop() {
				DetachResult::Detaching
			} else {
				DetachResult::AlreadyDetached
			}
		}
	}

	/// fetch a list of all buffers in a workspace
	pub async fn fetch_buffers(&self) -> crate::Result<()> {
		let mut workspace_client = self.0.services.workspace.clone();
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
		let mut workspace_client = self.0.services.workspace.clone();
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
			self.0.users.insert(u, UserInfo { uuid: u });
		}

		Ok(())
	}

	/// get a list of the users attached to a specific buffer
	///
	/// TODO: discuss implementation details
	pub async fn list_buffer_users(&self, path: &str) -> crate::Result<Vec<Identity>> {
		let mut workspace_client = self.0.services.workspace.clone();
		let buffer_users = workspace_client
			.list_buffer_users(tonic::Request::new(BufferNode {
				path: path.to_string(),
			}))
			.await?
			.into_inner()
			.users;

		Ok(buffer_users)
	}

	/// delete a buffer
	pub async fn delete(&self, path: &str) -> crate::Result<()> {
		let mut workspace_client = self.0.services.workspace.clone();
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
		self.0.buffers.iter().map(|elem| elem.key().clone()).collect()
	}

	/// get the currently cached "filetree"
	// #[cfg_attr(feature = "js", napi)] // https://github.com/napi-rs/napi-rs/issues/1120
	pub fn filetree(&self) -> Vec<String> {
		self.0.filetree.iter().map(|f| f.clone()).collect()
	}
}

impl Drop for WorkspaceInner {
	fn drop(&mut self) {
		for entry in self.buffers.iter() {
			if !entry.value().stop() {
				tracing::warn!("could not stop buffer worker {} for workspace {}", entry.value().name(), self.id);
			}
		}
		if !self.cursor.stop() {
			tracing::warn!("could not stop cursor worker for workspace {}", self.id);
		}
	}
}

pub enum DetachResult {
	NotAttached,
	Detaching,
	AlreadyDetached,
}
