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
	auth::{LoginRequest, auth_client::AuthClient},
	common::{Empty, Identifier, Token},
	session::{
		InviteRequest, OwnedWorkspaceRequest, session_client::SessionClient,
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
	user: Arc<User>,
	config: crate::api::Config,
	workspaces: DashMap<uuid::Uuid, Workspace>,
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
	pub async fn create_workspace(
		&self,
		name: impl AsRef<str>,
	) -> RemoteResult<crate::api::WorkspaceInfo> {
		let info = self
			.0
			.session
			.clone()
			.create_workspace(OwnedWorkspaceRequest {
				name: name.as_ref().to_string(),
			})
			.await?
			.into_inner();
		Ok(crate::api::WorkspaceInfo::from(info))
	}

	/// Delete an existing workspace if possible.
	pub async fn delete_workspace(&self, id: uuid::Uuid) -> RemoteResult<()> {
		self.0
			.session
			.clone()
			.delete_workspace(Identifier::from(id))
			.await?;
		Ok(())
	}

	/// Invite user with given username to the given workspace, if possible.
	pub async fn invite_to_workspace(
		&self,
		workspace_id: uuid::Uuid,
		user_name: impl AsRef<str>,
	) -> RemoteResult<()> {
		self.0
			.session
			.clone()
			.invite_to_workspace(InviteRequest {
				workspace: Identifier::from(workspace_id),
				user: user_name.as_ref().to_string(),
			})
			.await?;
		Ok(())
	}

	/// Fetch the names of all workspaces owned by the current user.
	pub async fn fetch_owned_workspaces(&self) -> RemoteResult<Vec<crate::api::WorkspaceInfo>> {
		Ok(self
			.0
			.session
			.clone()
			.fetch_owned_workspaces(Empty {})
			.await?
			.into_inner()
			.owned
			.into_iter()
			.map(crate::api::WorkspaceInfo::from)
			.collect())
	}

	/// Fetch the names of all workspaces the current user has joined.
	pub async fn fetch_joined_workspaces(&self) -> RemoteResult<Vec<crate::api::WorkspaceInfo>> {
		Ok(self
			.0
			.session
			.clone()
			.fetch_invited_workspaces(Empty {})
			.await?
			.into_inner()
			.invited
			.into_iter()
			.map(crate::api::WorkspaceInfo::from)
			.collect())
	}

	/// Join and return a [`Workspace`].
	#[tracing::instrument(skip(self, workspace), fields(ws = %workspace))]
	pub async fn attach_workspace(&self, workspace: uuid::Uuid) -> ConnectionResult<Workspace> {
		let mut session_client = self.0.session.clone();
		let token = session_client
			.get_workspace_token(Identifier::from(workspace))
			.await?
			.into_inner();

		let workspace_claims = InternallyMutable::new(token);

		let ws = Workspace::connect(
			workspace,
			self.0.user.clone(),
			self.0.config.clone(),
			workspace_claims.channel(),
			self.0.claims.channel(),
		)
		.await?;
		self.0.workspaces.insert(workspace, ws.clone());
		let mut workspace_client = ws.services().ws();

		let weak = Arc::downgrade(&ws.0);
		tokio::spawn(async move {
			let fut = async move {
				loop {
					// TODO either configurable token refresh time or calculate depending on token lifetime
					tokio::time::sleep(std::time::Duration::from_secs(240)).await;
					if weak.upgrade().is_none() { break };
					let new_credentials = session_client.get_workspace_token(
						tonic::Request::new(Identifier::from(workspace))
					)
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
	pub fn leave_workspace(&self, id: uuid::Uuid) -> bool {
		match self.0.workspaces.remove(&id) {
			None => true,
			Some(x) => x.1.consume(),
		}
	}

	/// Gets a [`Workspace`] handle by name.
	pub fn get_workspace(&self, id: uuid::Uuid) -> Option<Workspace> {
		self.0.workspaces.get(&id).map(|x| x.clone())
	}

	/// Get the names of all active [`Workspace`]s.
	pub fn active_workspaces(&self) -> Vec<uuid::Uuid> {
		self.0
			.workspaces
			.iter()
			.map(|x| *x.key())
			.collect()
	}

	/// Get the currently logged in user.
	pub fn current_user(&self) -> &User {
		&self.0.user
	}
}
