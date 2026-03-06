//! ### Client
//! Main `codemp` client, containing and managing all underlying services.

use std::sync::Arc;

use dashmap::DashMap;
use tonic::{
	service::interceptor::InterceptedService,
	transport::{Channel, Endpoint},
};

use crate::{
	api::UserInfo,
	errors::{ConnectionResult, RemoteResult},
	ext::InternallyMutable,
	network,
	workspace::Workspace,
};
use codemp_proto::{
	auth::{LoginRequest, auth_client::AuthClient},
	common::{Empty, Token},
	session::{
		InviteRequest, OwnedWorkspaceIdentifier, session_client::SessionClient,
	},
};

#[cfg(feature = "py")]
use pyo3::prelude::*;

/// A `codemp` client handle.
///
/// It generates a new UUID and stores user credentials upon connecting.
///
/// A new [`Client`] can be obtained with [`Client::connect`].
#[derive(Debug, Clone)]
#[cfg_attr(feature = "js", napi_derive::napi)]
#[cfg_attr(feature = "py", pyclass)]
pub struct Client(Arc<ClientInner>);

#[derive(Debug)]
struct ClientInner {
	user: Arc<UserInfo>,
	config: crate::api::Config,
	workspaces: DashMap<crate::api::WorkspaceIdentifier, Workspace>,
	auth: AuthClient<Channel>,
	session: SessionClient<InterceptedService<Channel, network::SessionInterceptor>>,
	claims: InternallyMutable<Token>,
}

impl Client {
	/// Connect to the server, authenticate and instantiate a new [`Client`].
	#[tracing::instrument]
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
			user: Arc::new(resp.user.into()),
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
	pub async fn create_workspace(&self, name: String) -> RemoteResult<()> {
		self.0
			.session
			.clone()
			.create_workspace(OwnedWorkspaceIdentifier { workspace: name })
			.await?
			.into_inner();
		Ok(())
	}

	/// Delete an existing workspace if possible.
	pub async fn delete_workspace(&self, name: String) -> RemoteResult<()> {
		self.0
			.session
			.clone()
			.delete_workspace(OwnedWorkspaceIdentifier { workspace: name })
			.await?;
		Ok(())
	}

	/// Invite user with given username to the given workspace, if possible.
	pub async fn invite_to_workspace(&self, workspace_name: String, user_name: String) -> RemoteResult<()> {
		self.0
			.session
			.clone()
			.invite_to_workspace(InviteRequest {
				workspace: workspace_name,
				user: user_name,
			})
			.await?;
		Ok(())
	}

	/// Fetch the names of all workspaces owned by the current user.
	pub async fn fetch_owned_workspaces(&self) -> RemoteResult<Vec<crate::api::WorkspaceIdentifier>> {
		Ok(self
			.0
			.session
			.clone()
			.fetch_owned_workspaces(Empty {})
			.await?
			.into_inner()
			.workspaces
			.into_iter()
			.map(crate::api::WorkspaceIdentifier::from)
			.collect())
	}

	/// Fetch the names of all workspaces the current user has joined.
	pub async fn fetch_joined_workspaces(&self) -> RemoteResult<Vec<crate::api::WorkspaceIdentifier>> {
		Ok(self
			.0
			.session
			.clone()
			.fetch_invited_workspaces(Empty {})
			.await?
			.into_inner()
			.workspaces
			.into_iter()
			.map(crate::api::WorkspaceIdentifier::from)
			.collect())
	}

	/// Join and return a [`Workspace`].
	#[tracing::instrument(skip(self, workspace), fields(ws = %workspace))]
	pub async fn attach_workspace(&self, workspace: crate::api::WorkspaceIdentifier) -> ConnectionResult<Workspace> {
		let mut session_client = self.0.session.clone();
		let token = session_client
			.get_workspace_token(codemp_proto::session::WorkspaceIdentifier::from(workspace.clone()))
			.await?
			.into_inner();

		let workspace_claims = InternallyMutable::new(token);

		let ws = Workspace::connect(
			workspace.clone(),
			self.0.user.clone(),
			self.0.config.clone(),
			workspace_claims.channel(),
			self.0.claims.channel(),
		)
		.await?;
		self.0.workspaces.insert(workspace.clone(), ws.clone());
		let mut workspace_client = ws.services().ws();

		let weak = Arc::downgrade(&ws.0);
		tokio::spawn(async move {
			let _workspace = workspace.clone();
			let fut = async move {
				loop {
					// TODO either configurable token refresh time or calculate depending on token lifetime
					tokio::time::sleep(std::time::Duration::from_secs(240)).await;
					if weak.upgrade().is_none() { break };
					let new_credentials = session_client.get_workspace_token(codemp_proto::session::WorkspaceIdentifier::from(_workspace.clone()))
						.await?
						.into_inner();
					workspace_claims.set(new_credentials);
					workspace_client.keep_alive(tonic::Request::new(Empty {})).await?;
				}
				Ok::<(), tonic::Status>(())
			};

			if let Err(e) = fut.await {
				tracing::error!("error in keepalive task for workspace {workspace}: {e}");
			}
		});

		Ok(ws)
	}

	/// Leave the [`Workspace`] with the given name.
	pub fn leave_workspace(&self, id: &crate::api::WorkspaceIdentifier) -> bool {
		match self.0.workspaces.remove(id) {
			None => true,
			Some(x) => x.1.consume(),
		}
	}

	/// Gets a [`Workspace`] handle by name.
	pub fn get_workspace(&self, id: &crate::api::WorkspaceIdentifier) -> Option<Workspace> {
		self.0.workspaces.get(id).map(|x| x.clone())
	}

	/// Get the names of all active [`Workspace`]s.
	pub fn active_workspaces(&self) -> Vec<crate::api::WorkspaceIdentifier> {
		self.0
			.workspaces
			.iter()
			.map(|x| x.value().id().clone())
			.collect()
	}

	/// Get the currently logged in user.
	pub fn current_user(&self) -> &UserInfo {
		&self.0.user
	}
}
