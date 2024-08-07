//! ### client
//!
//! codemp client manager, containing grpc services

use std::sync::Arc;

use dashmap::DashMap;
use tokio::sync::mpsc;
use tonic::service::interceptor::InterceptedService;
use tonic::service::Interceptor;
use tonic::transport::{Channel, Endpoint};
use tonic::IntoRequest;
use uuid::Uuid;

use codemp_proto::auth::auth_client::AuthClient;
use codemp_proto::{
	common::Empty,
	buffer::buffer_client::BufferClient,
	cursor::cursor_client::CursorClient,
	auth::{Token, WorkspaceJoinRequest},
	workspace::workspace_client::WorkspaceClient,
};
use crate::{
	api::controller::ControllerWorker,
	cursor::worker::CursorWorker,
	workspace::Workspace
};

/// codemp client manager
///
/// contains all required grpc services and the unique user id
/// will disconnect when dropped
/// can be used to interact with server
pub struct Client {
	user_id: Uuid,
	workspaces: DashMap<String, Workspace>,
	token_tx: Arc<tokio::sync::watch::Sender<Token>>,
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
	pub(crate) workspace: WorkspaceClient<InterceptedService<Channel, ClientInterceptor>>,
	pub(crate) buffer: BufferClient<InterceptedService<Channel, ClientInterceptor>>,
	pub(crate) cursor: CursorClient<InterceptedService<Channel, ClientInterceptor>>,
	pub(crate) auth: AuthClient<Channel>,
}

// TODO meno losco
fn parse_codemp_connection_string(string: &str) -> (String, String) {
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
		let auth = AuthClient::new(channel);

		let user_id = uuid::Uuid::new_v4();

		Ok(Client {
			user_id,
			workspaces: DashMap::default(),
			token_tx: Arc::new(token_tx),
			services: Arc::new(Services { workspace, buffer, cursor, auth })
		})
	}

	pub async fn login(&self, username: String, password: String, workspace_id: Option<String>) -> crate::Result<()> {
		Ok(self.token_tx.send(
			self.services.auth.clone()
				.login(WorkspaceJoinRequest { username, password, workspace_id})
				.await?
				.into_inner()
		)?)
	}

	/// join a workspace, returns an [tokio::sync::RwLock] to interact with it
	pub async fn join_workspace(&self, workspace: &str) -> crate::Result<Workspace> {
		let ws_stream = self.services.workspace.clone().attach(Empty{}.into_request()).await?.into_inner();

		let (tx, rx) = mpsc::channel(256);
		let cur_stream = self.services.cursor.clone()
			.attach(tokio_stream::wrappers::ReceiverStream::new(rx))
			.await?
			.into_inner();

		let worker = CursorWorker::default();
		let controller = worker.subscribe();
		tokio::spawn(async move {
			tracing::debug!("controller worker started");
			worker.work(tx, cur_stream).await;
			tracing::debug!("controller worker stopped");
		});

		let ws = Workspace::new(
			workspace.to_string(),
			self.user_id,
			controller,
			self.token_tx.clone(),
			self.services.clone()
		);

		ws.fetch_users().await?;
		ws.fetch_buffers().await?;

		ws.run_actor(ws_stream);

		self.workspaces.insert(workspace.to_string(), ws.clone());

		Ok(ws)
	}

	/// leaves a [Workspace] by name
	pub fn leave_workspace(&self, id: &str) -> bool {
		self.workspaces.remove(id).is_some()
	}

	/// gets a [Workspace] by name
	pub fn get_workspace(&self, id: &str) -> Option<Workspace> {
		self.workspaces.get(id).map(|x| x.clone())
	}

	/// accessor for user id
	pub fn user_id(&self) -> Uuid {
		self.user_id
	}
}
