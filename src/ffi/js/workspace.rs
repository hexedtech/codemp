use std::sync::Arc;

use napi_derive::napi;

use crate::ffi::js::{JsCodempError, buffer::JsBufferController, cursor::JsCursorController};


#[napi]
/// a reference to a codemp workspace
pub struct JsWorkspace(Arc<crate::Workspace>);

impl From<Arc<crate::Workspace>> for JsWorkspace {
	fn from(value: Arc<crate::Workspace>) -> Self {
		JsWorkspace(value)
	}
}

#[napi]
impl JsWorkspace {

	#[napi]
	pub fn id(&self) -> String {
		self.0.id()
	}
	
	#[napi]
	pub fn filetree(&self) -> Vec<String> {
		self.0.filetree()
	}

	#[napi]
	pub fn cursor(&self) -> JsCursorController {
		JsCursorController::from(self.0.cursor())
	}

	#[napi]
	pub fn buffer_by_name(&self, path: String) -> Option<JsBufferController> {
		self.0.buffer_by_name(&path).map(|b| JsBufferController::from(b))
	}

	#[napi]
	pub async fn create(&self, path: String) -> napi::Result<()> {
		Ok(self.0.create(&path).await.map_err(JsCodempError)?)
	}

	#[napi]
	pub async fn attach(&self, path: String) -> napi::Result<JsBufferController> {
		Ok(JsBufferController::from(self.0.attach(&path).await.map_err(JsCodempError)?))
	}
	
	/*#[napi]
	pub async fn delete(&self, path: String) -> napi::Result<>{
		self.0.delete(&path)
	}*/

}