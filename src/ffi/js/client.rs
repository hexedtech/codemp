use napi_derive::napi;
use crate::ffi::js::JsCodempError;
use crate::ffi::js::workspace::JsWorkspace;

#[napi]
/// main codemp client session
pub struct JsCodempClient(tokio::sync::RwLock<crate::Client>);

#[napi]
/// connect to codemp servers and return a client session
pub async fn connect(addr: Option<String>) -> napi::Result<JsCodempClient>{
	let client = crate::Client::new(addr.as_deref().unwrap_or("http://codemp.alemi.dev:50053"))
		.await
		.map_err(JsCodempError)?;

	Ok(JsCodempClient(tokio::sync::RwLock::new(client)))
}

#[napi]
impl JsCodempClient {
	#[napi]
	/// login against AuthService with provided credentials, optionally requesting access to a workspace
	pub async fn login(&self, username: String, password: String, workspace_id: Option<String>) -> napi::Result<()> {
		self.0.read().await.login(username, password, workspace_id).await.map_err(JsCodempError)?;
		Ok(())
	}

	#[napi]
	/// join workspace with given id (will start its cursor controller)
	pub async fn join_workspace(&self, workspace: String) -> napi::Result<JsWorkspace> {
		Ok(JsWorkspace::from(self.0.write().await.join_workspace(&workspace).await.map_err(JsCodempError)?))
	}

	#[napi]
	/// get workspace with given id, if it exists
	pub async fn get_workspace(&self, workspace: String) -> Option<JsWorkspace> {
		self.0.read().await.get_workspace(&workspace).map(|w| JsWorkspace::from(w))
	}

	#[napi]
	/// return current sessions's user id
	pub async fn user_id(&self) -> String {
		self.0.read().await.user_id().to_string()
	}
}