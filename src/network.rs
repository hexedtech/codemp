use codemp_proto::{
	common::Token, buffer::buffer_client::BufferClient, cursor::cursor_client::CursorClient,
	workspace::workspace_client::WorkspaceClient,
};
use tonic::{
	service::{interceptor::InterceptedService, Interceptor},
	transport::{Channel, Endpoint},
};

use crate::errors::ConnectionResult;

type AuthedService = InterceptedService<Channel, WorkspaceInterceptor>;

#[derive(Debug, Clone)]
pub struct SessionInterceptor(pub tokio::sync::watch::Receiver<codemp_proto::common::Token>);
impl tonic::service::Interceptor for SessionInterceptor {
	fn call(
		&mut self,
		mut request: tonic::Request<()>,
	) -> tonic::Result<tonic::Request<()>> {
		if let Ok(token) = self.0.borrow().token.parse() {
			request.metadata_mut().insert("session", token);
		}

		Ok(request)
	}
}

#[derive(Debug)]
pub struct Services {
	workspace: WorkspaceClient<AuthedService>,
	buffer: BufferClient<AuthedService>,
	cursor: CursorClient<AuthedService>,
}

impl Services {
	pub async fn try_new(
		dest: &str,
		session: tokio::sync::watch::Receiver<codemp_proto::common::Token>,
		workspace: tokio::sync::watch::Receiver<codemp_proto::common::Token>,
	) -> ConnectionResult<Self> {
		// TRANSPORT ERROR
		let channel = Endpoint::from_shared(dest.to_string())?.connect().await?;
		let inter = WorkspaceInterceptor { session, workspace };
		Ok(Self {
			cursor: CursorClient::with_interceptor(channel.clone(), inter.clone()),
			workspace: WorkspaceClient::with_interceptor(channel.clone(), inter.clone()),
			// TODO technically we could keep buffers on separate servers, and thus manage buffer
			// connections separately, but for now it's more convenient to bundle them with workspace
			buffer: BufferClient::with_interceptor(channel.clone(), inter.clone()),
		})
	}

	// TODO just make fields pub(crate) ?? idk
	pub fn ws(&self) -> WorkspaceClient<AuthedService> {
		self.workspace.clone()
	}

	pub fn buf(&self) -> BufferClient<AuthedService> {
		self.buffer.clone()
	}

	pub fn cur(&self) -> CursorClient<AuthedService> {
		self.cursor.clone()
	}
}

#[derive(Clone)]
pub struct WorkspaceInterceptor {
	session: tokio::sync::watch::Receiver<Token>,
	workspace: tokio::sync::watch::Receiver<Token>,
}

impl Interceptor for WorkspaceInterceptor {
	fn call(
		&mut self,
		mut request: tonic::Request<()>,
	) -> Result<tonic::Request<()>, tonic::Status> {
		if let Ok(token) = self.session.borrow().token.parse() {
			request.metadata_mut().insert("session", token);
		}

		if let Ok(token) = self.workspace.borrow().token.parse() {
			request.metadata_mut().insert("workspace", token);
		}

		Ok(request)
	}
}
