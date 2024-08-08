//! ### client
//!
//! codemp client manager, containing grpc services

use std::sync::Arc;

use dashmap::DashMap;
use tonic::transport::{Channel, Endpoint};
use uuid::Uuid;

use codemp_proto::auth::auth_client::AuthClient;
use codemp_proto::auth::{Token, WorkspaceJoinRequest};
use crate::workspace::Workspace;

#[derive(Debug)]
pub struct AuthWrap {
	username: String,
	password: String,
	service: AuthClient<Channel>,
}

impl AuthWrap {
	async fn try_new(username: &str, password: &str, host: &str) -> crate::Result<Self> {
		let channel = Endpoint::from_shared(host.to_string())?
			.connect()
			.await?;

		Ok(AuthWrap {
			username: username.to_string(),
			password: password.to_string(),
			service: AuthClient::new(channel),
		})
	}

	async fn login_workspace(&self, ws: &str) -> crate::Result<Token> {
		Ok(
			self.service.clone()
				.login(WorkspaceJoinRequest {
					username: self.username.clone(),
					password: self.password.clone(),
					workspace_id: Some(ws.to_string())
				})
				.await?
				.into_inner()
		)
	}
}

/// codemp client manager
///
/// contains all required grpc services and the unique user id
/// will disconnect when dropped
/// can be used to interact with server
#[derive(Debug, Clone)]
pub struct Client(Arc<ClientInner>);

#[derive(Debug)]
struct ClientInner {
	user_id: Uuid,
	host: String,
	workspaces: DashMap<String, Workspace>,
	auth: AuthWrap,
}

impl Client {
	/// instantiate and connect a new client
	pub async fn new(
		host: impl AsRef<str>,
		username: impl AsRef<str>,
		password: impl AsRef<str>
	) -> crate::Result<Self> {
		let host = if host.as_ref().starts_with("http") {
			host.as_ref().to_string()
		} else {
			format!("https://{}", host.as_ref())
		};

		let user_id = uuid::Uuid::new_v4();
		let auth = AuthWrap::try_new(username.as_ref(), password.as_ref(), &host).await?;

		Ok(Client(Arc::new(ClientInner {
			user_id,
			host,
			workspaces: DashMap::default(),
			auth,
		})))
	}

	/// join a workspace, returns an [tokio::sync::RwLock] to interact with it
	pub async fn join_workspace(&self, workspace: impl AsRef<str>) -> crate::Result<Workspace> {
		let token = self.0.auth.login_workspace(workspace.as_ref()).await?;

		let ws = Workspace::try_new(
			workspace.as_ref().to_string(),
			self.0.user_id,
			&self.0.host,
			token.clone()
		).await?;

		self.0.workspaces.insert(workspace.as_ref().to_string(), ws.clone());

		Ok(ws)
	}

	/// leaves a [Workspace] by name
	pub fn leave_workspace(&self, id: &str) -> bool {
		self.0.workspaces.remove(id).is_some()
	}

	/// gets a [Workspace] by name
	pub fn get_workspace(&self, id: &str) -> Option<Workspace> {
		self.0.workspaces.get(id).map(|x| x.clone())
	}

	/// accessor for user id
	pub fn user_id(&self) -> Uuid {
		self.0.user_id
	}
}
