//! ### Client
//! Main `codemp` client, containing and managing all underlying services.

use std::sync::Arc;

use dashmap::DashMap;
use tonic::{
	service::interceptor::InterceptedService,
	transport::{Channel, Endpoint},
};

use crate::{
	api::AsyncReceiver,
	errors::{ConnectionResult, RemoteResult},
	ext::{IgnorableError, InternallyMutable},
	network,
	workspace::Workspace,
};
use codemp_proto::{
	auth::{LoginRequest, auth_client::AuthClient},
	common::{Empty, Token, UserInfo},
	session::{
		InviteRequest, OwnedWorkspaceIdentifier, SessionEvent, SessionEventKind, UserId, WorkspaceIdentifier, session_client::SessionClient
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
#[cfg_attr(feature = "py", pyclass(from_py_object))]
pub struct Client(Arc<ClientInner>);

#[derive(Debug)]
struct ClientInner {
	user: Arc<codemp_proto::common::UserInfo>,
	config: crate::api::Config,
	workspaces: DashMap<String, DashMap<String, Workspace>>,
	auth: AuthClient<Channel>,
	session: SessionClient<InterceptedService<Channel, network::SessionInterceptor>>,
	claims: InternallyMutable<Token>,
	poll_tx: tokio::sync::mpsc::UnboundedSender<tokio::sync::oneshot::Sender<()>>,
	callback:
		tokio::sync::watch::Sender<Option<crate::api::controller::ControllerCallback<Client>>>,
	events: tokio::sync::Mutex<
		tokio::sync::mpsc::UnboundedReceiver<crate::proto::session::SessionEvent>,
	>,
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
		let mut session =
			SessionClient::with_interceptor(channel, network::SessionInterceptor(claims.channel()));

		let (ev_tx, ev_rx) = tokio::sync::mpsc::unbounded_channel();
		let (poll_tx, poll_rx) = tokio::sync::mpsc::unbounded_channel();
		let (cb_tx, cb_rx) = tokio::sync::watch::channel(None);

		let stream = session.attach(Empty {}).await?.into_inner();

		let worker = ClientWorker {
			callback: cb_rx,
			pollers: Vec::new(),
			poll_rx,
			events: ev_tx,
		};

		let inner = Arc::new(ClientInner {
			user: Arc::new(resp.user),
			workspaces: DashMap::default(),
			poll_tx,
			events: tokio::sync::Mutex::new(ev_rx),
			claims,
			auth,
			session,
			config,
			callback: cb_tx,
		});

		let weak = Arc::downgrade(&inner);
		let _t = tokio::spawn(async move {
			worker.work(stream, weak).await;
		});

		Ok(Client(inner))
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
	pub async fn create_workspace(&self, name: impl ToString) -> RemoteResult<()> {
		self.0
			.session
			.clone()
			.create_workspace(OwnedWorkspaceIdentifier {
				workspace: name.to_string(),
			})
			.await?
			.into_inner();
		Ok(())
	}

	/// Delete an existing workspace if possible.
	pub async fn delete_workspace(&self, name: impl ToString) -> RemoteResult<()> {
		self.0
			.session
			.clone()
			.delete_workspace(OwnedWorkspaceIdentifier {
				workspace: name.to_string(),
			})
			.await?;
		Ok(())
	}

	/// Quit a joined workspace. Cannot quit owned workspaces: must delete them
	pub async fn quit_workspace(
		&self,
		user: impl ToString,
		workspace: impl ToString,
	) -> RemoteResult<()> {
		self.0
			.session
			.clone()
			.quit_workspace(WorkspaceIdentifier {
				user: user.to_string(),
				workspace: workspace.to_string(),
			})
			.await?;
		Ok(())
	}

	/// Accept an invitation to a workspace, making it accessible
	pub async fn accept_invite(
		&self,
		user: impl ToString,
		workspace: impl ToString,
	) -> RemoteResult<()> {
		self.0
			.session
			.clone()
			.accept_invite(WorkspaceIdentifier {
				user: user.to_string(),
				workspace: workspace.to_string(),
			})
			.await?;
		Ok(())
	}

	/// Reject an invitation to a workspace
	pub async fn reject_invite(
		&self,
		user: impl ToString,
		workspace: impl ToString,
	) -> RemoteResult<()> {
		self.0
			.session
			.clone()
			.reject_invite(WorkspaceIdentifier {
				user: user.to_string(),
				workspace: workspace.to_string(),
			})
			.await?;
		Ok(())
	}

	/// Invite user with given username to the given workspace, if possible.
	pub async fn invite_to_workspace(
		&self,
		workspace_name: impl ToString,
		user_name: impl ToString,
	) -> RemoteResult<()> {
		self.0
			.session
			.clone()
			.invite_to_workspace(InviteRequest {
				workspace: workspace_name.to_string(),
				user: user_name.to_string(),
			})
			.await?;
		Ok(())
	}

	/// Fetch the names of all workspaces owned by the current user.
	pub async fn fetch_owned_workspaces(
		&self,
	) -> RemoteResult<Vec<WorkspaceIdentifier>> {
		Ok(self
			.0
			.session
			.clone()
			.fetch_owned_workspaces(Empty {})
			.await?
			.into_inner()
			.workspaces
		)
	}

	/// Fetch the names of all workspaces the current user has joined.
	pub async fn fetch_joined_workspaces(
		&self,
	) -> RemoteResult<Vec<WorkspaceIdentifier>> {
		Ok(self
			.0
			.session
			.clone()
			.fetch_invited_workspaces(Empty {})
			.await?
			.into_inner()
			.workspaces
		)
	}

	/// Get the meta information for a user
	pub async fn get_user_info(&self, user: impl ToString) -> RemoteResult<UserInfo> {
		Ok(self
			.0
			.session
			.clone()
			.get_user_info(UserId {
				user: user.to_string(),
			})
			.await?
			.into_inner()
		)
	}

	/// Join and return a [`Workspace`].
	#[tracing::instrument(skip(self, user, workspace), fields(owner = user.to_string(), ws = workspace.to_string()))]
	pub async fn attach_workspace(
		&self,
		user: impl ToString,
		workspace: impl ToString,
	) -> ConnectionResult<Workspace> {
		let workspace_id = WorkspaceIdentifier {
			user: user.to_string(),
			workspace: workspace.to_string(),
		};
		let user = user.to_string();
		let workspace = workspace.to_string();
		let mut session_client = self.0.session.clone();
		let token = session_client
			.get_workspace_token(workspace_id.clone())
			.await?
			.into_inner();

		let workspace_claims = InternallyMutable::new(token);

		let ws = Workspace::connect(
			workspace_id.clone(),
			self.0.user.clone(),
			self.0.config.clone(),
			workspace_claims.channel(),
			self.0.claims.channel(),
		)
		.await?;

		match self.0.workspaces.get_mut(&user) {
			Some(mutref) => {
				mutref.insert(workspace.clone(), ws.clone());
			}
			None => {
				let map = DashMap::default();
				map.insert(workspace.clone(), ws.clone());
				self.0.workspaces.insert(user.clone(), map);
			}
		};

		let mut workspace_client = ws.services().ws();

		let weak = Arc::downgrade(&ws.0);
		tokio::spawn(async move {
			let fut = async move {
				loop {
					// TODO either configurable token refresh time or calculate depending on token lifetime
					tokio::time::sleep(std::time::Duration::from_secs(240)).await;
					if weak.upgrade().is_none() {
						break;
					};
					let new_credentials = session_client
						.get_workspace_token(workspace_id.clone())
						.await?
						.into_inner();
					workspace_claims.set(new_credentials);
					workspace_client
						.keep_alive(tonic::Request::new(Empty {}))
						.await?;
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
	pub fn leave_workspace(&self, user: impl AsRef<str>, workspace: impl AsRef<str>) -> bool {
		if let Some(intermediate) = self.0.workspaces.get_mut(user.as_ref()) {
			if let Some((_name, ws)) = intermediate.remove(workspace.as_ref()) {
				let count = Arc::strong_count(&ws.0);
				tracing::debug!("there are {} more references to this workspace", count - 1);
				if Arc::into_inner(ws.0).is_some() {
					return true;
				}
			}
		}

		true // leaving a workspace we aren't attached to is a no-op: return true because no more refs
	}

	/// Gets a [`Workspace`] handle by name.
	pub fn get_workspace(
		&self,
		user: impl AsRef<str>,
		workspace: impl AsRef<str>,
	) -> Option<Workspace> {
		self.0
			.workspaces
			.get(user.as_ref())?
			.get(workspace.as_ref())
			.map(|x| x.clone())
	}

	/// Get the names of all active [`Workspace`]s.
	// TODO get rid of WorkspaceIdentifier
	pub fn active_workspaces(&self) -> Vec<WorkspaceIdentifier> {
		let mut out = Vec::new();
		for wss in self.0.workspaces.iter() {
			for ws in wss.value().iter() {
				out.push(ws.value().id().clone());
			}
		}
		out
	}

	/// Get the currently logged in user.
	pub fn current_user(&self) -> &UserInfo {
		&self.0.user
	}
}

impl AsyncReceiver<SessionEvent> for Client {
	async fn try_recv(
		&self,
	) -> crate::errors::ControllerResult<Option<SessionEvent>> {
		match self.0.events.lock().await.try_recv() {
			Ok(x) => Ok(Some(x)),
			Err(tokio::sync::mpsc::error::TryRecvError::Empty) => Ok(None),
			Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
				Err(crate::errors::ControllerError::Stopped)
			}
		}
	}

	async fn poll(&self) -> crate::errors::ControllerResult<()> {
		let (tx, rx) = tokio::sync::oneshot::channel();
		self.0.poll_tx.send(tx)?;
		Ok(rx.await?)
	}

	fn clear_callback(&self) {
		self.0.callback.send_replace(None);
	}

	fn callback(&self, cb: impl Into<crate::api::controller::ControllerCallback<Self>>) {
		self.0.callback.send_replace(Some(cb.into()));
	}
}

struct ClientWorker {
	callback:
		tokio::sync::watch::Receiver<Option<crate::api::controller::ControllerCallback<Client>>>,
	pollers: Vec<tokio::sync::oneshot::Sender<()>>,
	poll_rx: tokio::sync::mpsc::UnboundedReceiver<tokio::sync::oneshot::Sender<()>>,
	events: tokio::sync::mpsc::UnboundedSender<SessionEvent>,
}

impl ClientWorker {
	#[tracing::instrument(skip(self, stream, weak))]
	pub(crate) async fn work(
		mut self,
		mut stream: tonic::Streaming<SessionEvent>,
		weak: std::sync::Weak<ClientInner>,
	) {
		tracing::debug!("client worker starting");
		loop {
			if weak.upgrade().is_none() {
				break tracing::debug!("client worker clean exit");
			}

			tokio::select! {
				_ = tokio::time::sleep(std::time::Duration::from_secs(30)) => {},

				res = self.poll_rx.recv() => match res {
				None => break tracing::debug!("pollers channel closed: client has been dropped"),
					Some(x) => self.pollers.push(x),
				},

				res = stream.message() => match res {
					Err(e) => break tracing::error!("client stream closed: {e}"),
					Ok(None) => break tracing::info!("closing client"),
					Ok(Some(event)) => {
						let Some(_inner) = weak.upgrade() else {
							break tracing::debug!("client worker clean exit");
						};
						tracing::debug!("received client event: {event:?}");
						match event.kind() {
							SessionEventKind::InvitationEvent => {
								tracing::info!("got invited to workspace {}/{} by {}", event.workspace.user, event.workspace.workspace, event.user);
							},
							SessionEventKind::QuitEvent => {
								tracing::info!("user {} left workspace {}/{}", event.user, event.workspace.user, event.workspace.workspace);
							},
							SessionEventKind::AcceptEvent => {
								tracing::info!("user {} accepted invite to workspace {}/{}", event.user, event.workspace.user, event.workspace.workspace);
							},
							SessionEventKind::RejectEvent => {
								tracing::info!("user {} rejected invite to workspace {}/{}", event.user, event.workspace.user, event.workspace.workspace);
							},
						}

						if self.events.send(event).is_err() {
							tracing::warn!("no active controller to receive client event");
						}
						self.pollers.drain(..).for_each(|x| {
							x.send(()).unwrap_or_warn("poller dropped before completion");
						});
						if let Some(cb) = self.callback.borrow().as_ref() {
							if let Some(ws) = weak.upgrade() {
								cb.call(Client(ws));
							} else {
								break tracing::debug!("client worker clean (late) exit");
							}
						}
					}
				},
			}
		}
		tracing::debug!("workspace worker stopping");
	}
}
