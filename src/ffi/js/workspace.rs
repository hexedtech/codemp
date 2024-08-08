use std::sync::Arc;

use napi_derive::napi;
use crate::prelude::*;


#[napi]
impl CodempWorkspace {
	#[napi(js_name = "id")]
	pub fn js_id(&self) -> String {
		self.id()
	}
	
	#[napi(js_name = "filetree")]
	pub fn js_filetree(&self) -> Vec<String> {
		self.filetree()
	}

	#[napi(js_name = "cursor")]
	pub fn js_cursor(&self) -> CodempCursorController {
		self.cursor()
	}

	#[napi(js_name = "buffer_by_name")]
	pub fn js_buffer_by_name(&self, path: String) -> Option<CodempBufferController> {
		self.buffer_by_name(&path)
	}

	#[napi(js_name = "create")]
	pub async fn js_create(&self, path: String) -> napi::Result<()> {
		Ok(self.create(&path).await?)
	}

	#[napi(js_name = "attach")]
	pub async fn js_attach(&self, path: String) -> napi::Result<CodempBufferController> {
		Ok(self.attach(&path).await?)
	}
	
	#[napi(js_name = "delete")]
	pub async fn js_delete(&self, path: String) -> napi::Result<()> {
		Ok(self.delete(&path).await?)
	}

}