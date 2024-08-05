use std::{collections::BTreeSet, sync::Arc};
use tokio::sync::mpsc;
use dashmap::{DashMap, DashSet};
use tonic::Streaming;
use uuid::Uuid;
use crate::{
	api::controller::ControllerWorker, buffer::{self, worker::BufferWorker}, client::Services, cursor,
};
use codemp_proto::{auth::Token, common::{Identity, Empty}, files::BufferNode, workspace::{WorkspaceEvent, workspace_event::{Event as WorkspaceEventInner, FileCreate, FileDelete, FileRename, UserJoin, UserLeave}}};

//TODO may contain more info in the future
#[derive(Debug, Clone)]
pub struct UserInfo {
	pub uuid: Uuid
}

#[derive(Debug, Clone)]
pub struct Workspace(Arc<WorkspaceInner>);

#[derive(Debug)]
struct WorkspaceInner {
	id: String,
	user_id: Uuid, // reference to global user id
	token: Arc<tokio::sync::watch::Sender<Token>>,
	cursor: cursor::Controller,
	buffers: Arc<DashMap<String, buffer::Controller>>,
	pub(crate) filetree: Arc<DashSet<String>>,
	pub(crate) users: Arc<DashMap<Uuid, UserInfo>>,
	services: Arc<Services>
}

impl Workspace {
	/// create a new buffer and perform initial fetch operations
	pub(crate) fn new(
		id: String,
		user_id: Uuid,
		token: Arc<tokio::sync::watch::Sender<Token>>,
		cursor: cursor::Controller,
		services: Arc<Services>
	) -> Self {
		Self(Arc::new(WorkspaceInner {
			id,
			user_id,
			token,
			cursor,
			buffers: Arc::new(DashMap::default()),
			filetree: Arc::new(DashSet::default()),
			users: Arc::new(DashMap::default()),
			services
		}))
	}

	pub(crate) fn run_actor(&self, mut stream: Streaming<WorkspaceEvent>) {
		let users = self.0.users.clone();
		let filetree = self.0.filetree.clone();
		let name = self.id();
		tokio::spawn(async move {
			loop {
				match stream.message().await {
					Err(e) => break tracing::error!("workspace '{}' stream closed: {}", name, e),
					Ok(None) => break tracing::info!("leaving workspace {}", name),
					Ok(Some(WorkspaceEvent { event: None })) => tracing::warn!("workspace {} received empty event", name),
					Ok(Some(WorkspaceEvent { event: Some(ev) })) => match ev {
						WorkspaceEventInner::Join(UserJoin { user }) => { users.insert(user.clone().into(), UserInfo { uuid: user.into() }); },
						WorkspaceEventInner::Leave(UserLeave { user }) => { users.remove(&user.into()); },
						WorkspaceEventInner::Create(FileCreate { path }) => { filetree.insert(path); },
						WorkspaceEventInner::Rename(FileRename { before, after }) => { filetree.remove(&before); filetree.insert(after); },
						WorkspaceEventInner::Delete(FileDelete { path }) => { filetree.remove(&path); },
					},
				}
			}
		});
	}

	/// create a new buffer in current workspace
	pub async fn create(&self, path: &str) -> crate::Result<()> {
		let mut workspace_client = self.0.services.workspace.clone();
		workspace_client.create_buffer(
			tonic::Request::new(BufferNode { path: path.to_string() })
		).await?;

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
		let request = tonic::Request::new(BufferNode { path: path.to_string() });
		let credentials = worskspace_client.access_buffer(request).await?.into_inner();
		self.0.token.send(credentials.token)?;

		let (tx, rx) = mpsc::channel(256);
		let mut req = tonic::Request::new(tokio_stream::wrappers::ReceiverStream::new(rx));
		req.metadata_mut().insert("path", tonic::metadata::MetadataValue::try_from(credentials.id.id).expect("could not represent path as byte sequence"));
		let stream = self.0.services.buffer.clone().attach(req).await?.into_inner();

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

	/// fetch a list of all buffers in a workspace
	pub async fn fetch_buffers(&self) -> crate::Result<()> {
		let mut workspace_client = self.0.services.workspace.clone();
		let buffers = workspace_client.list_buffers(
			tonic::Request::new(Empty {})
		).await?.into_inner().buffers;

		self.0.filetree.clear();
		for b in buffers {
			self.0.filetree.insert(b.path);
		}

		Ok(())
	}

	/// fetch a list of all users in a workspace
	pub async fn fetch_users(&self) -> crate::Result<()> {
		let mut workspace_client = self.0.services.workspace.clone();
		let users = BTreeSet::from_iter(workspace_client.list_users(
			tonic::Request::new(Empty {})
		).await?.into_inner().users.into_iter().map(Uuid::from));

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
		let buffer_users = workspace_client.list_buffer_users(
			tonic::Request::new(BufferNode { path: path.to_string() })
		).await?.into_inner().users;

		Ok(buffer_users)
	}
	
	/// delete a buffer
	pub async fn delete(&self, path: &str) -> crate::Result<()> {
		let mut workspace_client = self.0.services.workspace.clone();
		workspace_client.delete_buffer(
			tonic::Request::new(BufferNode { path: path.to_string() })
		).await?;
	
		self.0.filetree.remove(path);
	
		Ok(())
	}

	/// get the id of the workspace
	pub fn id(&self) -> String { self.0.id.clone() }

	/// return a reference to current cursor controller, if currently in a workspace
	pub fn cursor(&self) -> cursor::Controller { self.0.cursor.clone() }

	/// get a new reference to a buffer controller, if any is active to given path
	pub fn buffer_by_name(&self, path: &str) -> Option<buffer::Controller> {
		self.0.buffers.get(path).map(|x| x.clone())
	}

	/// get the currently cached "filetree"
	pub fn filetree(&self) -> Vec<String> {
		self.0.filetree.iter().map(|f| f.clone()).collect()
	}
}
