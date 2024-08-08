use napi_derive::napi;
use crate::prelude::*;

#[napi]
/// connect to codemp servers and return a client session
pub async fn connect(addr: Option<String>, username: String, password: String) -> napi::Result<CodempClient>{
	let client = crate::Client::new(addr.as_deref().unwrap_or("http://codemp.alemi.dev:50053"), username, password)
		.await?;

	Ok(client)
}

#[napi]
impl CodempClient {
	#[napi(js_name = "join_workspace")]
	/// join workspace with given id (will start its cursor controller)
	pub async fn js_join_workspace(&self, workspace: String) -> napi::Result<CodempWorkspace> {
		Ok(self.join_workspace(workspace).await?)
	}

	#[napi(js_name = "get_workspace")]
	/// get workspace with given id, if it exists
	pub fn js_get_workspace(&self, workspace: String) -> Option<CodempWorkspace> {
		self.get_workspace(&workspace)
	}

	#[napi(js_name = "user_id")]
	/// return current sessions's user id
	pub fn js_user_id(&self) -> String {
		self.user_id().to_string()
	}

	#[napi(js_name = "active_workspaces")]
	pub fn js_active_workspaces(&self) -> Vec<String> {
		self.active_workspaces()
	}
}