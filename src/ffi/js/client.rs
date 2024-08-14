use napi_derive::napi;
use crate::{Client, Workspace};

#[napi]
/// connect to codemp servers and return a client session
pub async fn connect(addr: Option<String>, username: String, password: String) -> napi::Result<crate::Client>{
	let client = crate::Client::new(addr.as_deref().unwrap_or("http://codemp.alemi.dev:50053"), username, password)
		.await?;

	Ok(client)
}

#[napi]
impl Client {
	#[napi(js_name = "join_workspace")]
	/// join workspace with given id (will start its cursor controller)
	pub async fn js_join_workspace(&self, workspace: String) -> napi::Result<Workspace> {
		Ok(self.join_workspace(workspace).await?)
	}

	#[napi(js_name = "get_workspace")]
	/// get workspace with given id, if it exists
	pub fn js_get_workspace(&self, workspace: String) -> Option<Workspace> {
		self.get_workspace(&workspace)
	}

	#[napi(js_name = "user_id")]
	/// return current sessions's user id
	pub fn js_user_id(&self) -> String {
		self.user_id().to_string()
	}

	#[napi(js_name = "active_workspaces")]
	/// get list of all active workspaces
	pub fn js_active_workspaces(&self) -> Vec<String> {
		self.active_workspaces()
	}
}