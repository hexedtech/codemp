use crate::prelude::*;
use napi_derive::napi;

#[napi]
/// connect to codemp servers and return a client session
pub async fn connect(config: crate::api::Config) -> napi::Result<crate::Client> {
	Ok(crate::Client::connect(config).await?)
}

#[napi]
impl CodempClient {
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
	pub async fn js_fetch_owned_workspaces(&self) -> napi::Result<Vec<CodempWorkspaceIdentifier>> {
		Ok(self.fetch_owned_workspaces().await?)
	}

	#[napi(js_name = "fetchJoinedWorkspaces")]
	/// fetch joined workspaces
	pub async fn js_fetch_joined_workspaces(&self) -> napi::Result<Vec<CodempWorkspaceIdentifier>> {
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
	pub async fn js_attach_workspace(&self, user: String, workspace: String) -> napi::Result<CodempWorkspace> {
		Ok(self.attach_workspace(&user, &workspace).await?)
	}

	#[napi(js_name = "leaveWorkspace")]
	/// leave workspace and disconnect, returns true if workspace was active
	pub async fn js_leave_workspace(&self, user: String, workspace: String) -> bool {
		self.leave_workspace(&user, workspace)
	}

	#[napi(js_name = "getWorkspace")]
	/// get workspace with given id, if it exists
	pub fn js_get_workspace(&self, user: String, workspace: String) -> Option<CodempWorkspace> {
		self.get_workspace(&user, &workspace)
	}

	#[napi(js_name = "currentUser")]
	/// return current sessions's user id
	pub fn js_current_user(&self) -> CodempUserInfo {
		self.current_user().clone().into()
	}

	#[napi(js_name = "activeWorkspaces")]
	/// get list of all active workspaces
	pub fn js_active_workspaces(&self) -> Vec<CodempWorkspaceIdentifier> {
		self.active_workspaces()
	}

	#[napi(js_name = "refresh")]
	/// refresh client session token
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
	pub async fn js_get_user_info(&self, user: String) -> napi::Result<CodempUserInfo> {
		Ok(self.get_user_info(&user).await?.into())
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


}
