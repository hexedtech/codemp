use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi_derive::napi;

use crate::prelude::{
	CodempAsyncReceiver as AsyncReceiver,
	CodempConfig as Config,
	CodempSession as Session,
	CodempUserInfo as UserInfo,
	CodempSessionEvent as SessionEvent,
	CodempWorkspace as Workspace,
	CodempWorkspaceIdentifier as WorkspaceIdentifier,
};

/// connect to codemp servers and return a session
#[allow(dead_code)]
#[napi]
pub async fn connect(config: Config) -> napi::Result<Session> {
	Ok(Session::connect(config).await?)
}

#[napi]
impl Session {
	#[napi(js_name = "createWorkspace")]
	/// create workspace with given id, if able to
	pub async fn js_create_workspace(&self, workspace: String) -> napi::Result<()> {
		Ok(self.create_workspace(workspace).await?)
	}

	#[napi(js_name = "deleteWorkspace")]
	/// delete workspace with given id, if able to
	pub async fn js_delete_workspace(&self, workspace: String) -> napi::Result<()> {
		Ok(self.delete_workspace(workspace).await?)
	}

	#[napi(js_name = "fetchOwnedWorkspaces")]
	/// fetch owned workspaces
	pub async fn js_fetch_owned_workspaces(&self) -> napi::Result<Vec<WorkspaceIdentifier>> {
		Ok(self.fetch_owned_workspaces().await?)
	}

	#[napi(js_name = "fetchJoinedWorkspaces")]
	/// fetch joined workspaces
	pub async fn js_fetch_joined_workspaces(&self) -> napi::Result<Vec<WorkspaceIdentifier>> {
		Ok(self.fetch_joined_workspaces().await?)
	}

	#[napi(js_name = "inviteToWorkspace")]
	/// invite user to given workspace, if able to
	pub async fn js_invite_to_workspace(
		&self,
		workspace: String,
		user: String,
	) -> napi::Result<()> {
		Ok(self.invite_to_workspace(workspace, user).await?)
	}

	#[napi(js_name = "attachWorkspace")]
	/// join workspace with given id (will start its cursor controller)
	pub async fn js_attach_workspace(&self, user: String, workspace: String) -> napi::Result<Workspace> {
		Ok(self.attach_workspace(&user, &workspace).await?)
	}

	#[napi(js_name = "leaveWorkspace")]
	/// leave workspace and disconnect, returns true if workspace was active
	pub fn js_leave_workspace(&self, user: String, workspace: String) -> bool {
		self.leave_workspace(&user, workspace)
	}

	#[napi(js_name = "getWorkspace")]
	/// get workspace with given id, if it exists
	pub fn js_get_workspace(&self, user: String, workspace: String) -> Option<Workspace> {
		self.get_workspace(&user, &workspace)
	}

	#[napi(js_name = "currentUser")]
	/// return current sessions's user id
	pub fn js_current_user(&self) -> UserInfo {
		self.current_user().clone()
	}

	#[napi(js_name = "activeWorkspaces")]
	/// get list of all active workspaces
	pub fn js_active_workspaces(&self) -> Vec<WorkspaceIdentifier> {
		self.active_workspaces()
	}

	#[napi(js_name = "refresh")]
	/// refresh session token
	pub async fn js_refresh(&self) -> napi::Result<()> {
		Ok(self.refresh().await?)
	}

	/// Accept an invitation to a workspace, making it accessible
	#[napi(js_name = "acceptInvite")]
	pub async fn js_accept_invite(&self, user: String, workspace: String) -> napi::Result<()> {
		Ok(self.accept_invite(&user, &workspace).await?)
	}

	/// Get the meta information for a user
	#[napi(js_name = "getUserInfo")]
	pub async fn js_get_user_info(&self, user: String) -> napi::Result<UserInfo> {
		Ok(self.get_user_info(&user).await?)
	}

	/// Quit a joined workspace. Cannot quit owned workspaces: must delete them
	#[napi(js_name = "quitWorkspace")]
	pub async fn js_quit_workspace(&self, user: String, workspace: String) -> napi::Result<()> {
		Ok(self.quit_workspace(&user, &workspace).await?)
	}

	/// Reject an invitation to a workspace
	#[napi(js_name = "rejectInvite")]
	pub async fn js_reject_invite(&self, user: String, workspace: String) -> napi::Result<()> {
		Ok(self.reject_invite(&user, &workspace).await?)
	}

	/// Register a callback to be called on receive.
	/// There can only be one callback registered at any given time.
	#[napi(
		js_name = "callback",
		ts_args_type = "fun: (err: Error|null, event: Session) => void"
	)]
	pub fn js_callback(
		&self,
		fun: ThreadsafeFunction<Session>,
	) -> napi::Result<()> {
		self.callback(move |controller: Session| {
			fun.call(Ok(controller.clone()), ThreadsafeFunctionCallMode::Blocking);
			//check this with tracing also we could use Ok(event) to get the error
			// If it blocks the main thread too many time we have to change this
		});

		Ok(())
	}

	/// Clear the registered callback
	#[napi(js_name = "clearCallback")]
	pub fn js_clear_callback(&self) {
		self.clear_callback();
	}

	/// Get next session event if available without blocking
	#[napi(js_name = "tryRecv")]
	pub async fn js_try_recv(&self) -> napi::Result<Option<SessionEvent>> {
		Ok(self.try_recv().await?)
	}

	/// Block until next session event
	#[napi(js_name = "recv")]
	pub async fn js_recv(&self) -> napi::Result<SessionEvent> {
		Ok(self.recv().await?)
	}

	/// Block until next session event without returning it
	#[napi(js_name = "poll")]
	pub async fn js_poll(&self) -> napi::Result<()> {
		Ok(self.poll().await?)
	}
}
