use crate::buffer::controller::BufferController;
use crate::cursor::controller::CursorController;
use crate::ffi::js::client::JsUser;
use crate::Workspace;
use napi_derive::napi;

#[napi(object, js_name = "Event")]
pub struct JsEvent {
	pub r#type: String,
	pub value: String,
}

impl From<crate::api::Event> for JsEvent {
	fn from(value: crate::api::Event) -> Self {
		match value {
			crate::api::Event::FileTreeUpdated(value) => Self {
				r#type: "filetree".into(),
				value,
			},
			crate::api::Event::UserJoin(value) => Self {
				r#type: "join".into(),
				value,
			},
			crate::api::Event::UserLeave(value) => Self {
				r#type: "leave".into(),
				value,
			},
		}
	}
}

#[napi]
impl Workspace {
	/// Get the unique workspace id
	#[napi(js_name = "id")]
	pub fn js_id(&self) -> String {
		self.id()
	}

	/// List all available buffers in this workspace
	#[napi(js_name = "filetree")]
	pub fn js_filetree(&self, filter: Option<&str>, strict: bool) -> Vec<String> {
		self.filetree(filter, strict)
	}

	/// List all user names currently in this workspace
	#[napi(js_name = "user_list")]
	pub fn js_user_list(&self) -> Vec<String> {
		self.user_list()
	}

	/// List all currently active buffers
	#[napi(js_name = "buffer_list")]
	pub fn js_buffer_list(&self) -> Vec<String> {
		self.buffer_list()
	}

	/// Get workspace's Cursor Controller
	#[napi(js_name = "cursor")]
	pub fn js_cursor(&self) -> CursorController {
		self.cursor()
	}

	/// Get a buffer controller by its name (path)
	#[napi(js_name = "buffer_by_name")]
	pub fn js_buffer_by_name(&self, path: String) -> Option<BufferController> {
		self.buffer_by_name(&path)
	}

	/// Create a new buffer in the current workspace
	#[napi(js_name = "create")]
	pub async fn js_create(&self, path: String) -> napi::Result<()> {
		Ok(self.create(&path).await?)
	}

	/// Attach to a workspace buffer, starting a BufferController
	#[napi(js_name = "attach")]
	pub async fn js_attach(&self, path: String) -> napi::Result<BufferController> {
		Ok(self.attach(&path).await?)
	}

	/// Delete a buffer from workspace
	#[napi(js_name = "delete")]
	pub async fn js_delete(&self, path: String) -> napi::Result<()> {
		Ok(self.delete(&path).await?)
	}

	/// Detach from an active buffer, stopping its underlying worker
	/// this method returns true if no reference or last reference was held, false if there are still
	/// dangling references to clear
	#[napi(js_name = "detach")]
	pub async fn js_detach(&self, path: String) -> bool {
		self.detach(&path)
	}

	/// Wait for next workspace event and return it
	#[napi(js_name = "event")]
	pub async fn js_event(&self) -> napi::Result<JsEvent> {
		Ok(JsEvent::from(self.event().await?))
	}

	/// Re-fetch remote buffer list
	#[napi(js_name = "fetch_buffers")]
	pub async fn js_fetch_buffers(&self) -> napi::Result<()> {
		Ok(self.fetch_buffers().await?)
	}
	/// Re-fetch the list of all users in the workspace.
	#[napi(js_name = "fetch_users")]
	pub async fn js_fetch_users(&self) -> napi::Result<()> {
		Ok(self.fetch_users().await?)
	}

	/// List users attached to a specific buffer
	#[napi(js_name = "list_buffer_users")]
	pub async fn js_list_buffer_users(&self, path: String) -> napi::Result<Vec<JsUser>> {
		Ok(self
			.list_buffer_users(&path)
			.await?
			.into_iter()
			.map(JsUser::from)
			.collect())
	}
}
