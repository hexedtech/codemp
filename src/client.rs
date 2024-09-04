//! ### Client
//! Main `codemp` client, containing and managing all underlying services.

use std::sync::Arc;

use dashmap::DashMap;
use tonic::{service::interceptor::InterceptedService, transport::{Channel, Endpoint}};

use crate::{api::User, errors::{ConnectionResult, RemoteResult}, ext::InternallyMutable, network, workspace::Workspace};
use codemp_proto::{
	auth::{auth_client::AuthClient, LoginRequest},
	common::{Empty, Token}, session::{session_client::SessionClient, InviteRequest, WorkspaceRequest},
};

#[cfg(feature = "python")]
use pyo3::prelude::*;

/// A `codemp` client handle.
///
/// It generates a new UUID and stores user credentials upon connecting.
///
/// A new [`Client`] can be obtained with [`Client::connect`].
#[derive(Debug, Clone)]
#[cfg_attr(feature = "js", napi_derive::napi)]
#[cfg_attr(feature = "python", pyclass)]
pub struct Client(Arc<ClientInner>);

#[derive(Debug)]
struct ClientInner {
	user: User,
	host: String,
	workspaces: DashMap<String, Workspace>,
	auth: AuthClient<Channel>,
	session: SessionClient<InterceptedService<Channel, network::SessionInterceptor>>,
	claims: InternallyMutable<Token>,
}

impl Client {
	/// Connect to the server, authenticate and instantiate a new [`Client`].
	pub async fn connect(
		host: impl AsRef<str>,
		username: impl AsRef<str>,
		password: impl AsRef<str>,
	) -> ConnectionResult<Self> {
		let host = if host.as_ref().starts_with("http") {
			host.as_ref().to_string()
		} else {
			format!("https://{}", host.as_ref())
		};

		let channel = Endpoint::from_shared(host.clone())?.connect().await?;
		let mut auth = AuthClient::new(channel.clone());

		let resp = auth.login(LoginRequest {
			username: username.as_ref().to_string(),
			password: password.as_ref().to_string(),
		})
			.await?
			.into_inner();

		let claims = InternallyMutable::new(resp.token);

		let session = SessionClient::with_interceptor(
			channel, network::SessionInterceptor(claims.channel())
		);

		Ok(Client(Arc::new(ClientInner {
			host,
			user: resp.user.into(),
			workspaces: DashMap::default(),
			claims,
			auth, session,
		})))
	}

	/// Refresh session token.
	pub async fn refresh(&self) -> RemoteResult<()> {
		let new_token = self.0.auth.clone().refresh(self.0.claims.get())
			.await?
			.into_inner();
		self.0.claims.set(new_token);
		Ok(())
	}

	/// Attempt to create a new workspace with given name.
	pub async fn create_workspace(&self, name: impl AsRef<str>) -> RemoteResult<()> {
		self.0.session
			.clone()
			.create_workspace(WorkspaceRequest { workspace: name.as_ref().to_string() })
			.await?;
		Ok(())
	}

	/// Delete an existing workspace if possible.
	pub async fn delete_workspace(&self, name: impl AsRef<str>) -> RemoteResult<()> {
		self.0.session
			.clone()
			.delete_workspace(WorkspaceRequest { workspace: name.as_ref().to_string() })
			.await?;
		Ok(())
	}

	/// Invite user with given username to the given workspace, if possible.
	pub async fn invite_to_workspace(&self, workspace_name: impl AsRef<str>, user_name: impl AsRef<str>) -> RemoteResult<()> {
		self.0.session
			.clone()
			.invite_to_workspace(InviteRequest {
				workspace: workspace_name.as_ref().to_string(),
				user: user_name.as_ref().to_string(),
			})
			.await?;
		Ok(())
	}

	/// List all available workspaces, also filtering between those owned and those invited to.
	pub async fn list_workspaces(&self, owned: bool, invited: bool) -> RemoteResult<Vec<String>> {
		let mut workspaces = self.0.session
			.clone()
			.list_workspaces(Empty {})
			.await?
			.into_inner();

		let mut out = Vec::new();

		if owned { out.append(&mut workspaces.owned) }
		if invited { out.append(&mut workspaces.invited) }

		Ok(out)
	}

	/// Join and return a [`Workspace`].
	pub async fn join_workspace(&self, workspace: impl AsRef<str>) -> ConnectionResult<Workspace> {
		let token = self.0.session
			.clone()
			.access_workspace(WorkspaceRequest { workspace: workspace.as_ref().to_string() })
			.await?
			.into_inner();

		let ws = Workspace::try_new(
			workspace.as_ref().to_string(),
			self.0.user.clone(),
			&self.0.host,
			token,
			self.0.claims.channel(),
		)
		.await?;

		self.0
			.workspaces
			.insert(workspace.as_ref().to_string(), ws.clone());

		Ok(ws)
	}

	/// Leave the [`Workspace`] with the given name.
	pub fn leave_workspace(&self, id: &str) -> bool {
		self.0.workspaces.remove(id).is_some()
	}

	/// Gets a [`Workspace`] handle by name.
	pub fn get_workspace(&self, id: &str) -> Option<Workspace> {
		self.0.workspaces.get(id).map(|x| x.clone())
	}

	/// Get the names of all active [`Workspace`]s.
	pub fn active_workspaces(&self) -> Vec<String> {
		self.0
			.workspaces
			.iter()
			.map(|x| x.key().to_string())
			.collect()
	}

	/// Get the currently logged in user.
	pub fn user(&self) -> &User {
		&self.0.user
	}
}
