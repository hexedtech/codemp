//! ### Client
//! Main `codemp` client, containing and managing all underlying services.

use std::sync::Arc;

use dashmap::DashMap;
use tonic::{
	service::interceptor::InterceptedService,
	transport::{Channel, Endpoint},
};

use crate::{
	api::User,
	errors::{ConnectionResult, RemoteResult},
	ext::InternallyMutable,
	network,
	workspace::Workspace,
};
use codemp_proto::{
	auth::{auth_client::AuthClient, LoginRequest},
	common::{Empty, Token},
	session::{session_client::SessionClient, InviteRequest, WorkspaceRequest},
};

#[cfg(any(feature = "py", feature = "py-noabi"))]
use pyo3::prelude::*;

/// A `codemp` client handle.
///
/// It generates a new UUID and stores user credentials upon connecting.
///
/// A new [`Client`] can be obtained with [`Client::connect`].
#[derive(Debug, Clone)]
#[cfg_attr(feature = "js", napi_derive::napi)]
#[cfg_attr(any(feature = "py", feature = "py-noabi"), pyclass)]
pub struct Client(Arc<ClientInner>);

#[derive(Debug)]
struct ClientInner {
	user: User,
	config: crate::api::Config,
	workspaces: DashMap<String, Workspace>,
	auth: AuthClient<Channel>,
	session: SessionClient<InterceptedService<Channel, network::SessionInterceptor>>,
	claims: InternallyMutable<Token>,
}

impl Client {
	/// Connect to the server, authenticate and instantiate a new [`Client`].
	pub async fn connect(config: crate::api::Config) -> ConnectionResult<Self> {
		// TODO move these two into network.rs
		let channel = Endpoint::from_shared(config.endpoint())?.connect().await?;
		let mut auth = AuthClient::new(channel.clone());

		let resp = auth
			.login(LoginRequest {
				username: config.username.clone(),
				password: config.password.clone(),
			})
			.await?
			.into_inner();

		let claims = InternallyMutable::new(resp.token);

		// TODO move this one into network.rs
		let session =
			SessionClient::with_interceptor(channel, network::SessionInterceptor(claims.channel()));

		Ok(Client(Arc::new(ClientInner {
			user: resp.user.into(),
			workspaces: DashMap::default(),
			claims,
			auth,
			session,
			config,
		})))
	}

	/// Refresh session token.
	pub async fn refresh(&self) -> RemoteResult<()> {
		let new_token = self
			.0
			.auth
			.clone()
			.refresh(self.0.claims.get())
			.await?
			.into_inner();
		self.0.claims.set(new_token);
		Ok(())
	}

	/// Attempt to create a new workspace with given name.
	pub async fn create_workspace(&self, name: impl AsRef<str>) -> RemoteResult<()> {
		self.0
			.session
			.clone()
			.create_workspace(WorkspaceRequest {
				workspace: name.as_ref().to_string(),
			})
			.await?;
		Ok(())
	}

	/// Delete an existing workspace if possible.
	pub async fn delete_workspace(&self, name: impl AsRef<str>) -> RemoteResult<()> {
		self.0
			.session
			.clone()
			.delete_workspace(WorkspaceRequest {
				workspace: name.as_ref().to_string(),
			})
			.await?;
		Ok(())
	}

	/// Invite user with given username to the given workspace, if possible.
	pub async fn invite_to_workspace(
		&self,
		workspace_name: impl AsRef<str>,
		user_name: impl AsRef<str>,
	) -> RemoteResult<()> {
		self.0
			.session
			.clone()
			.invite_to_workspace(InviteRequest {
				workspace: workspace_name.as_ref().to_string(),
				user: user_name.as_ref().to_string(),
			})
			.await?;
		Ok(())
	}

	/// Fetch the names of all workspaces owned by the current user.
	pub async fn fetch_owned_workspaces(&self) -> RemoteResult<Vec<String>> {
		self.fetch_workspaces(true).await
	}

	/// Fetch the names of all workspaces the current user has joined.
	pub async fn fetch_joined_workspaces(&self) -> RemoteResult<Vec<String>> {
		self.fetch_workspaces(false).await
	}

	async fn fetch_workspaces(&self, owned: bool) -> RemoteResult<Vec<String>> {
		let workspaces = self
			.0
			.session
			.clone()
			.list_workspaces(Empty {})
			.await?
			.into_inner();

		if owned {
			Ok(workspaces.owned)
		} else {
			Ok(workspaces.invited)
		}
	}

	/// Join and return a [`Workspace`].
	pub async fn attach_workspace(
		&self,
		workspace: impl AsRef<str>,
	) -> ConnectionResult<Workspace> {
		let token = self
			.0
			.session
			.clone()
			.access_workspace(WorkspaceRequest {
				workspace: workspace.as_ref().to_string(),
			})
			.await?
			.into_inner();

		let ws = Workspace::connect(
			workspace.as_ref().to_string(),
			self.0.user.clone(),
			self.0.config.clone(),
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
		match self.0.workspaces.remove(id) {
			None => true,
			Some(x) => x.1.consume(),
		}
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
	pub fn current_user(&self) -> &User {
		&self.0.user
	}
}
