use crate::{Client, Workspace};
use napi_derive::napi;

#[napi(object, js_name = "User")]
pub struct JsUser {
	pub uuid: String,
	pub name: String,
}

impl TryFrom<JsUser> for crate::api::User {
	type Error = <uuid::Uuid as std::str::FromStr>::Err;
	fn try_from(value: JsUser) -> Result<Self, Self::Error> {
		Ok(Self {
			id: value.uuid.parse()?,
			name: value.name,
		})
	}
}

impl From<crate::api::User> for JsUser {
	fn from(value: crate::api::User) -> Self {
		Self {
			uuid: value.id.to_string(),
			name: value.name,
		}
	}
}

#[napi]
/// connect to codemp servers and return a client session
pub async fn connect(config: crate::api::Config) -> napi::Result<crate::Client> {
	Ok(crate::Client::connect(config).await?)
}

#[napi]
impl Client {
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
	pub async fn js_fetch_owned_workspaces(&self) -> napi::Result<Vec<String>> {
		Ok(self.fetch_owned_workspaces().await?)
	}

	#[napi(js_name = "fetchJoinedWorkspaces")]
	/// fetch joined workspaces
	pub async fn js_fetch_joined_workspaces(&self) -> napi::Result<Vec<String>> {
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
	pub async fn js_attach_workspace(&self, workspace: String) -> napi::Result<Workspace> {
		Ok(self.attach_workspace(workspace).await?)
	}

	#[napi(js_name = "leaveWorkspace")]
	/// leave workspace and disconnect, returns true if workspace was active
	pub async fn js_leave_workspace(&self, workspace: String) -> bool {
		self.leave_workspace(&workspace)
	}

	#[napi(js_name = "getWorkspace")]
	/// get workspace with given id, if it exists
	pub fn js_get_workspace(&self, workspace: String) -> Option<Workspace> {
		self.get_workspace(&workspace)
	}

	#[napi(js_name = "currentUser")]
	/// return current sessions's user id
	pub fn js_current_user(&self) -> JsUser {
		self.current_user().clone().into()
	}

	#[napi(js_name = "activeWorkspaces")]
	/// get list of all active workspaces
	pub fn js_active_workspaces(&self) -> Vec<String> {
		self.active_workspaces()
	}

	#[napi(js_name = "refresh")]
	/// refresh client session token
	pub async fn js_refresh(&self) -> napi::Result<()> {
		Ok(self.refresh().await?)
	}
}
