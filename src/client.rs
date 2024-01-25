//! ### client
//!
//! codemp client manager, containing grpc services

use std::collections::BTreeMap;
use std::sync::Arc;

use tokio::sync::{mpsc, RwLock};
use tonic::service::interceptor::InterceptedService;
use tonic::service::Interceptor;
use tonic::transport::{Channel, Endpoint};
use uuid::Uuid;

use crate::api::controller::ControllerWorker;
use crate::cursor::worker::CursorWorker;
use crate::proto::buffer_service::buffer_client::BufferClient;
use crate::proto::cursor_service::cursor_client::CursorClient;
use crate::proto::workspace::{JoinRequest, Token, WorkspaceDetails};
use crate::proto::workspace_service::workspace_client::WorkspaceClient;
use crate::workspace::Workspace;

/// codemp client manager
///
/// contains all required grpc services and the unique user id
/// will disconnect when dropped
/// can be used to interact with server
pub struct Client {
	user_id: Uuid,
	token_tx: Arc<tokio::sync::watch::Sender<Token>>,
	pub workspaces: BTreeMap<String, Arc<RwLock<Workspace>>>,
	services: Arc<Services>
}

#[derive(Clone)]
pub(crate) struct ClientInterceptor {
	token: tokio::sync::watch::Receiver<Token>
}

impl Interceptor for ClientInterceptor {
	fn call(&mut self, mut request: tonic::Request<()>) -> Result<tonic::Request<()>, tonic::Status> {
		if let Ok(token) = self.token.borrow().token.parse() {
			request.metadata_mut().insert("auth", token);
		}
		
		Ok(request)
	}
}


#[derive(Debug, Clone)]
pub(crate) struct Services {
	pub(crate) workspace: crate::proto::workspace_service::workspace_client::WorkspaceClient<InterceptedService<Channel, ClientInterceptor>>,
	pub(crate) buffer: crate::proto::buffer_service::buffer_client::BufferClient<InterceptedService<Channel, ClientInterceptor>>,
	pub(crate) cursor: crate::proto::cursor_service::cursor_client::CursorClient<InterceptedService<Channel, ClientInterceptor>>,
}

// TODO meno losco
fn parse_codemp_connection_string<'a>(string: &'a str) -> (String, String) {
	let url = string.replace("codemp://", "");
	let (host, workspace) = url.split_once('/').unwrap();
	(format!("http://{}", host), workspace.to_string())
}

impl Client {
	/// instantiate and connect a new client
	pub async fn new(dest: &str) -> crate::Result<Self> {
		let (_host, _workspace_id) = parse_codemp_connection_string(dest);

		let channel = Endpoint::from_shared(dest.to_string())?
			.connect()
			.await?;

		let (token_tx, token_rx) = tokio::sync::watch::channel(
			Token { token: "".to_string() }
		);

		let inter = ClientInterceptor { token: token_rx };

		let buffer = BufferClient::with_interceptor(channel.clone(), inter.clone());
		let cursor = CursorClient::with_interceptor(channel.clone(), inter.clone());
		let workspace = WorkspaceClient::with_interceptor(channel.clone(), inter.clone());

		let user_id = uuid::Uuid::new_v4();

		Ok(Client {
			user_id,
			token_tx: Arc::new(token_tx),
			workspaces: BTreeMap::new(),
			services: Arc::new(Services { workspace, buffer, cursor })
		})
	}

	/// creates a new workspace (and joins it implicitly), returns an [tokio::sync::RwLock] to interact with it
	pub async fn create_workspace(&mut self, workspace_id: &str) -> crate::Result<Arc<RwLock<Workspace>>> {
		let mut workspace_client = self.services.workspace.clone();
		workspace_client.create_workspace(
			tonic::Request::new(WorkspaceDetails { id: workspace_id.to_string() })
		).await?;

		self.join_workspace(workspace_id).await
	}

	/// join a workspace, returns an [tokio::sync::RwLock] to interact with it
	pub async fn join_workspace(&mut self, workspace_id: &str) -> crate::Result<Arc<RwLock<Workspace>>> {
		self.token_tx.send(self.services.workspace.clone().join(
			tonic::Request::new(JoinRequest { username: "".to_string(), password: "".to_string() }) //TODO
		).await?.into_inner())?;

		let (tx, rx) = mpsc::channel(10);
		let stream = self.services.cursor.clone()
			.attach(tokio_stream::wrappers::ReceiverStream::new(rx))
			.await?
			.into_inner();

		let worker = CursorWorker::new(self.user_id.clone());
		let controller = Arc::new(worker.subscribe());
		tokio::spawn(async move {
			tracing::debug!("controller worker started");
			worker.work(tx, stream).await;
			tracing::debug!("controller worker stopped");
		});

		let lock = Arc::new(RwLock::new(
			Workspace::new(
				workspace_id.to_string(),
				self.user_id,
				self.token_tx.clone(),
				controller,
				self.services.clone()
			).await?
		));

		self.workspaces.insert(workspace_id.to_string(), lock.clone());

		Ok(lock)
	}

	/// leave given workspace, disconnecting buffer and cursor controllers
	pub async fn leave_workspace(&self, workspace_id: &str) -> crate::Result<()> {
		let mut workspace_client = self.services.workspace.clone();
		workspace_client.leave_workspace(
			tonic::Request::new(WorkspaceDetails { id: workspace_id.to_string() })
		).await?;
		
		Ok(())
	}

	/// accessor for user id
	pub fn user_id(&self) -> Uuid {
		self.user_id.clone()
	}
}
