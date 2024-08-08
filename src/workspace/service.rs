use codemp_proto::{auth::Token, buffer::buffer_client::BufferClient, cursor::cursor_client::CursorClient, workspace::workspace_client::WorkspaceClient};
use tonic::{service::{interceptor::InterceptedService, Interceptor}, transport::{Channel, Endpoint}};


#[derive(Clone)]
pub struct WorkspaceInterceptor {
	token: tokio::sync::watch::Receiver<Token>
}

impl Interceptor for WorkspaceInterceptor {
	fn call(&mut self, mut request: tonic::Request<()>) -> Result<tonic::Request<()>, tonic::Status> {
		if let Ok(token) = self.token.borrow().token.parse() {
			request.metadata_mut().insert("auth", token);
		}
		
		Ok(request)
	}
}

type AuthedService = InterceptedService<Channel, WorkspaceInterceptor>;

#[derive(Debug)]
pub struct Services {
	token: tokio::sync::watch::Sender<Token>,
	workspace: WorkspaceClient<AuthedService>,
	buffer: BufferClient<AuthedService>,
	cursor: CursorClient<AuthedService>,
}

impl Services {
	pub async fn try_new(dest: &str, token: Token) -> crate::Result<Self> {
		let channel = Endpoint::from_shared(dest.to_string())?
			.connect()
			.await?;
		let (token_tx, token_rx) = tokio::sync::watch::channel(token);
		let inter = WorkspaceInterceptor { token: token_rx };
		Ok(Self {
			token: token_tx,
			buffer: BufferClient::with_interceptor(channel.clone(), inter.clone()),
			cursor: CursorClient::with_interceptor(channel.clone(), inter.clone()),
			workspace: WorkspaceClient::with_interceptor(channel.clone(), inter.clone()),
		})
	}

	pub fn set_token(&self, token: Token) {
		if self.token.send(token).is_err() {
			tracing::warn!("could not update token: no more auth interceptors active");
		}
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
