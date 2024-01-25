use std::{collections::{BTreeMap, BTreeSet}, str::FromStr, sync::Arc};
use tokio::sync::mpsc;
use uuid::Uuid;
use crate::{
	proto::{user::UserIdentity, workspace::{AttachRequest, BufferListRequest, BufferPayload, Token, UserListRequest}},
	api::controller::ControllerWorker,
	buffer::{self, worker::BufferWorker},
	client::Services,
	cursor
};

//TODO may contain more info in the future
#[derive(Debug, Clone)]
pub struct UserInfo {
	pub uuid: Uuid
}

impl From<Uuid> for UserInfo {
	fn from(uuid: Uuid) -> Self {
		UserInfo {
			uuid
		}
	}
}

impl From<UserIdentity> for Uuid {
	fn from(uid: UserIdentity) -> Uuid {
		Uuid::from_str(&uid.id).expect("expected an uuid")
	}
}

/// list_users -> A() , B()
/// get_user_info(B) -> B(cacca, pipu@piu)

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

	/// create a new buffer in current workspace, with optional given content
	pub async fn create(&mut self, path: &str) -> crate::Result<()> {
		let mut workspace_client = self.services.workspace.clone();
		workspace_client.create(
			tonic::Request::new(BufferPayload { path: path.to_string() })
		).await?;

		//add to filetree
		self.filetree.insert(path.to_string());

		Ok(())
	}

	/// attach to a buffer, starting a buffer controller and returning a new reference to it
	///
	/// to interact with such buffer use [crate::api::Controller::send] or
	/// [crate::api::Controller::recv] to exchange [crate::api::TextChange]
	pub async fn attach(&mut self, path: &str) -> crate::Result<Arc<buffer::Controller>> {
		let mut worskspace_client = self.services.workspace.clone();
		self.token.send(worskspace_client.attach(
			tonic::Request::new(AttachRequest { id: path.to_string() })
		).await?.into_inner())?;

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

	pub async fn fetch_buffers(&mut self) -> crate::Result<()> {
		let mut workspace_client = self.services.workspace.clone();
		let buffers = workspace_client.list_buffers(
			tonic::Request::new(BufferListRequest {})
		).await?.into_inner().buffers;

		self.filetree.clear();
		for b in buffers {
			self.filetree.insert(b.path);
		}

		Ok(())
	}

	pub async fn fetch_users(&mut self) -> crate::Result<()> {
		let mut workspace_client = self.services.workspace.clone();
		let users = BTreeSet::from_iter(workspace_client.list_users(
			tonic::Request::new(UserListRequest {})
		).await?.into_inner().users.into_iter().map(Uuid::from));

		// only keep userinfo for users that still exist
		self.users.retain(|k, _v| users.contains(k));

		let _users = self.users.clone(); // damnnn rust
		users.iter()
			.filter(|u| _users.contains_key(u))
			.for_each(|u| { self.users.insert(*u, UserInfo::from(*u)); });
		
		Ok(())
	}

	pub async fn list_buffer_users() {
		todo!(); //TODO what is this
	}

	pub async fn delete(&mut self, path: &str) -> crate::Result<()> {
		let mut workspace_client = self.services.workspace.clone();
		workspace_client.delete(
			tonic::Request::new(BufferPayload { path: path.to_string() })
		).await?;

		self.filetree.remove(path);

		Ok(())
	}
	
	/// leave current workspace if in one, disconnecting buffer and cursor controllers
	pub fn leave_workspace(&self) {
		todo!(); //TODO need proto
	}
	
	/// disconnect from a specific buffer
	pub fn disconnect_buffer(&mut self, path: &str) -> bool {
		match &mut self.buffers.remove(path) {
			None => false,
			Some(_) => true
		}
	}

	pub fn id(&self) -> String { self.id.clone() }

	/// get a new reference to a buffer controller, if any is active to given path
	pub fn buffer_by_name(&self, path: &str) -> Option<Arc<buffer::Controller>> {
		self.buffers.get(path).cloned()
	}

	/// return a reference to current cursor controller, if currently in a workspace
	pub fn cursor(&self) -> Arc<cursor::Controller> { self.cursor.clone() }

}

/*
impl Interceptor for Workspace { //TODO
	fn call(&mut self, mut request: tonic::Request<()>) -> Result<tonic::Request<()>, tonic::Status> {
		request.metadata_mut().insert("auth", self.token.token.parse().unwrap());
		Ok(request)
	}
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum FSNode {
	File(String),
	Directory(String, Vec<FSNode>),
}
	fn file_tree_rec(path: &str, root: &mut Vec<FSNode>) {
		if let Some(idx) = path.find("/") {
			let dir = path[..idx].to_string();
			let mut dir_node = vec![];
			Self::file_tree_rec(&path[idx..], &mut dir_node);
			root.push(FSNode::Directory(dir, dir_node));
		} else {
			root.push(FSNode::File(path.to_string()));
		}
	}
	
	fn file_tree(&self) -> Vec<FSNode> {
		let mut root = vec![];
		for path in &self.filetree {
			Self::file_tree_rec(&path, &mut root);	
		}
		root
	}
*/