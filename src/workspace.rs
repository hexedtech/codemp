use std::{collections::{BTreeMap, BTreeSet}, str::FromStr, sync::Arc};
use tokio::sync::mpsc;
use uuid::Uuid;
use crate::{
	api::controller::ControllerWorker, buffer::{self, worker::BufferWorker}, client::Services, cursor,
	proto::{auth::Token, common::{Identity, Empty}, files::BufferNode, workspace::{WorkspaceEvent, workspace_event::{Event as WorkspaceEventInner, FileCreate, FileDelete, FileRename, UserJoin, UserLeave}}}
};

//TODO may contain more info in the future
#[derive(Debug, Clone)]
pub struct UserInfo {
	pub uuid: Uuid
}

pub struct Workspace {
	id: String,
	user_id: Uuid,
	token: Arc<tokio::sync::watch::Sender<Token>>,
	cursor: Arc<cursor::Controller>,
	buffers: BTreeMap<String, Arc<buffer::Controller>>,
	filetree: BTreeSet<String>,
	users: BTreeMap<Uuid, UserInfo>,
	services: Arc<Services>
}

impl Workspace {
	/// create a new buffer and perform initial fetch operations
	pub(crate) async fn new(
		id: String,
		user_id: Uuid,
		token: Arc<tokio::sync::watch::Sender<Token>>,
		cursor: Arc<cursor::Controller>,
		services: Arc<Services>
	) -> crate::Result<Self> {
		let mut ws = Workspace {
			id,
			user_id,
			token,
			cursor,
			buffers: BTreeMap::new(),
			filetree: BTreeSet::new(),
			users: BTreeMap::new(),
			services
		};

		ws.fetch_buffers().await?;
		ws.fetch_users().await?;

		Ok(ws)
	}

	/// create a new buffer in current workspace
	pub async fn create(&mut self, path: &str) -> crate::Result<()> {
		let mut workspace_client = self.services.workspace.clone();
		workspace_client.create_buffer(
			tonic::Request::new(BufferNode { path: path.to_string() })
		).await?;

		// add to filetree
		self.filetree.insert(path.to_string());

		// fetch buffers
		self.fetch_buffers().await?;

		Ok(())
	}

	/// attach to a buffer, starting a buffer controller and returning a new reference to it
	///
	/// to interact with such buffer use [crate::api::Controller::send] or
	/// [crate::api::Controller::recv] to exchange [crate::api::TextChange]
	pub async fn attach(&mut self, path: &str) -> crate::Result<Arc<buffer::Controller>> {
		let mut worskspace_client = self.services.workspace.clone();
		let mut request = tonic::Request::new(AttachRequest { path: path.to_string() });
		request.metadata_mut().insert("path", tonic::metadata::MetadataValue::try_from(path).expect("could not represent path as byte sequence"));
		self.token.send(worskspace_client.attach(request).await?.into_inner())?;

		let (tx, rx) = mpsc::channel(10);
		let stream = self.services.buffer.clone()
			.attach(tokio_stream::wrappers::ReceiverStream::new(rx))
			.await?
			.into_inner();

		let worker = BufferWorker::new(self.user_id, path);
		let controller = Arc::new(worker.subscribe());
		tokio::spawn(async move {
			tracing::debug!("controller worker started");
			worker.work(tx, stream).await;
			tracing::debug!("controller worker stopped");
		});
		
		self.buffers.insert(path.to_string(), controller.clone());

		Ok(controller)
	}

	/// fetch a list of all buffers in a workspace
	pub async fn fetch_buffers(&mut self) -> crate::Result<()> {
		let mut workspace_client = self.services.workspace.clone();
		let buffers = workspace_client.list_buffers(
			tonic::Request::new(Empty {})
		).await?.into_inner().buffers;

		self.filetree.clear();
		for b in buffers {
			self.filetree.insert(b.path);
		}

		Ok(())
	}

	/// fetch a list of all users in a workspace
	pub async fn fetch_users(&mut self) -> crate::Result<()> {
		let mut workspace_client = self.services.workspace.clone();
		let users = BTreeSet::from_iter(workspace_client.list_users(
			tonic::Request::new(Empty {})
		).await?.into_inner().users.into_iter().map(Uuid::from));

		// only keep userinfo for users that still exist
		self.users.retain(|k, _v| users.contains(k));

		let _users = self.users.clone(); // damnnn rust
		users.iter()
			.filter(|u| _users.contains_key(u))
			.for_each(|u| { self.users.insert(*u, UserInfo::from(*u)); });
		
		Ok(())
	}

	/// get a list of the users attached to a specific buffer
	/// 
	/// TODO: discuss implementation details
	pub async fn list_buffer_users(&mut self, path: &str) -> crate::Result<Vec<UserIdentity>> {
		let mut workspace_client = self.services.workspace.clone();
		let buffer_users = workspace_client.list_buffer_users(
			tonic::Request::new(BufferNode { path: path.to_string() })
		).await?.into_inner().users;

		Ok(buffer_users)
	}
	
	/// detach from a specific buffer, returns false if there
	pub fn detach(&mut self, path: &str) -> bool {
		match &mut self.buffers.remove(path) {
			None => false,	
			Some(_) => true
		}
	}

	/// delete a buffer
	pub async fn delete(&mut self, path: &str) -> crate::Result<()> {
		let mut workspace_client = self.services.workspace.clone();
		workspace_client.delete_buffer(
			tonic::Request::new(BufferNode { path: path.to_string() })
		).await?;
	
		self.filetree.remove(path);
	
		Ok(())
	}

	/// get the id of the workspace
	pub fn id(&self) -> String { self.id.clone() }

	/// return a reference to current cursor controller, if currently in a workspace
	pub fn cursor(&self) -> Arc<cursor::Controller> { self.cursor.clone() }

	/// get a new reference to a buffer controller, if any is active to given path
	pub fn buffer_by_name(&self, path: &str) -> Option<Arc<buffer::Controller>> {
		self.buffers.get(path).cloned()
	}

	/// get the currently cached "filetree"
	pub fn filetree(&self) -> Vec<String> {
		self.filetree.iter().map(|f| f.clone()).collect()
	} 
}