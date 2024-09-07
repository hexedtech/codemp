use napi_derive::napi;
use crate::{Client, Workspace};

#[napi]
/// connect to codemp servers and return a client session
pub async fn connect(addr: Option<String>, username: String, password: String) -> napi::Result<crate::Client>{
	let client = crate::Client::connect(addr.as_deref().unwrap_or("http://code.mp:50053"), username, password)
		.await?;

	Ok(client)
}

#[napi]
impl Client {
	#[napi(js_name = "create_workspace")]
	/// create workspace with given id, if able to
	pub async fn js_create_workspace(&self, workspace: String) -> napi::Result<()> {
		Ok(self.create_workspace(workspace).await?)
	}

	#[napi(js_name = "delete_workspace")]
	/// delete workspace with given id, if able to
	pub async fn js_delete_workspace(&self, workspace: String) -> napi::Result<()> {
		Ok(self.delete_workspace(workspace).await?)
	}

	#[napi(js_name = "list_workspaces")]
	/// list available workspaces
	pub async fn js_list_workspaces(&self, owned: bool, invited: bool) -> napi::Result<Vec<String>> {
		Ok(self.list_workspaces(owned, invited).await?)
	}

	#[napi(js_name = "invite_to_workspace")]
	/// invite user to given workspace, if able to
	pub async fn js_invite_to_workspace(&self, workspace: String, user: String) -> napi::Result<()> {
		Ok(self.invite_to_workspace(workspace, user).await?)
	}

	#[napi(js_name = "join_workspace")]
	/// join workspace with given id (will start its cursor controller)
	pub async fn js_join_workspace(&self, workspace: String) -> napi::Result<Workspace> {
		Ok(self.join_workspace(workspace).await?)
	}

	#[napi(js_name = "leave_workspace")]
	/// leave workspace and disconnect, returns true if workspace was active
	pub async fn js_leave_workspace(&self, workspace: String) -> napi::Result<bool> {
		Ok(self.leave_workspace(&workspace))
	}

	#[napi(js_name = "get_workspace")]
	/// get workspace with given id, if it exists
	pub fn js_get_workspace(&self, workspace: String) -> Option<Workspace> {
		self.get_workspace(&workspace)
	}

	#[napi(js_name = "user_id")]
	/// return current sessions's user id
	pub fn js_user_id(&self) -> String {
		self.user().id.to_string()
	}

	#[napi(js_name = "active_workspaces")]
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
