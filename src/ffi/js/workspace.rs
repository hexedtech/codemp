use napi_derive::napi;
use crate::Workspace;
use crate::buffer::controller::BufferController;
use crate::cursor::controller::CursorController;

#[napi(object, js_name = "Event")]
pub struct JsEvent {
	pub r#type: String,
	pub value: String,
}

impl From<crate::api::Event> for JsEvent {
	fn from(value: crate::api::Event) -> Self {
		match value {
			crate::api::Event::FileTreeUpdated(value) => Self { r#type: "filetree".into(), value },
			crate::api::Event::UserJoin(value) => Self { r#type: "join".into(), value },
			crate::api::Event::UserLeave(value) => Self { r#type: "leave".into(), value },
		}
	}
}

#[napi]
impl Workspace {
	#[napi(js_name = "id")]
	pub fn js_id(&self) -> String {
		self.id()
	}
	
	#[napi(js_name = "filetree")]
	pub fn js_filetree(&self, filter: Option<&str>, strict: bool) -> Vec<String> {
		self.filetree(filter, strict)
	}

	#[napi(js_name = "user_list")]
	pub fn js_user_list(&self) -> Vec<String> {
		self.user_list()
	}

	#[napi(js_name = "cursor")]
	pub fn js_cursor(&self) -> CursorController {
		self.cursor()
	}

	#[napi(js_name = "buffer_by_name")]
	pub fn js_buffer_by_name(&self, path: String) -> Option<BufferController> {
		self.buffer_by_name(&path)
	}

	#[napi(js_name = "create")]
	pub async fn js_create(&self, path: String) -> napi::Result<()> {
		Ok(self.create(&path).await?)
	}

	#[napi(js_name = "attach")]
	pub async fn js_attach(&self, path: String) -> napi::Result<BufferController> {
		Ok(self.attach(&path).await?)
	}
	
	#[napi(js_name = "delete")]
	pub async fn js_delete(&self, path: String) -> napi::Result<()> {
		Ok(self.delete(&path).await?)
	}

	#[napi(js_name = "event")]
	pub async fn js_event(&self) -> napi::Result<JsEvent> {
		Ok(JsEvent::from(self.event().await?))
	}
}
