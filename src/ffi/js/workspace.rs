use crate::prelude::*;
use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi_derive::napi;

#[napi]
impl CodempWorkspace {
	/// Get the unique workspace id
	#[napi(js_name = "id")]
	pub fn js_id(&self) -> CodempWorkspaceIdentifier {
		self.id().clone()
	}

	/// List all available buffers in this workspace
	#[napi(js_name = "searchBuffers")]
	pub fn js_search_buffers(&self, filter: Option<String>) -> Vec<String> {
		self.search_buffers(filter.as_deref())
	}

	/// List all user names currently in this workspace
	#[napi(js_name = "userList")]
	pub fn js_user_list(&self) -> Vec<CodempUserInfo> {
		self.user_list()
	}

	/// List all currently active buffers
	#[napi(js_name = "activeBuffers")]
	pub fn js_active_buffers(&self) -> Vec<String> {
		self.active_buffers()
	}

	/// Get workspace's Cursor Controller
	#[napi(js_name = "cursor")]
	pub fn js_cursor(&self) -> CodempCursorController {
		self.cursor()
	}

	/// Get a buffer controller by its name (path)
	#[napi(js_name = "getBuffer")]
	pub fn js_get_buffer(&self, path: String) -> Option<CodempBufferController> {
		self.get_buffer(&path)
	}

	/// Create a new buffer in the current workspace
	#[napi(js_name = "createBuffer")]
	pub async fn js_create_buffer(&self, path: String, ephemeral: bool) -> napi::Result<()> {
		Ok(self.create_buffer(&path, ephemeral).await?)
	}

	/// Attach to a workspace buffer, starting a BufferController
	#[napi(js_name = "attachBuffer")]
	pub async fn js_attach_buffer(&self, path: String) -> napi::Result<CodempBufferController> {
		Ok(self.attach_buffer(&path).await?)
	}

	/// Delete a buffer from workspace
	#[napi(js_name = "deleteBuffer")]
	pub async fn js_delete_buffer(&self, path: String) -> napi::Result<()> {
		Ok(self.delete_buffer(&path).await?)
	}

	#[napi(js_name = "recv")]
	pub async fn js_recv(&self) -> napi::Result<CodempWorkspaceEvent> {
		Ok(self.recv().await?)
	}

	#[napi(js_name = "tryRecv")]
	pub async fn js_try_recv(&self) -> napi::Result<Option<CodempWorkspaceEvent>> {
		Ok(self.try_recv().await?)
	}

	#[napi(js_name = "poll")]
	pub async fn js_poll(&self) -> napi::Result<()> {
		self.poll().await?;
		Ok(())
	}

	#[napi(js_name = "clearCallback")]
	pub fn js_clear_callback(&self) -> napi::Result<()> {
		self.clear_callback();
		Ok(())
	}

	#[napi(js_name = "callback", ts_args_type = "fun: (event: Workspace) => void")]
	pub fn js_callback(&self, fun: ThreadsafeFunction<CodempWorkspace>) -> napi::Result<()> {
		let tsfn: ThreadsafeFunction<CodempWorkspace> = fun;
		self.callback(move |controller: CodempWorkspace| {
			tsfn.call(Ok(controller.clone()), ThreadsafeFunctionCallMode::Blocking); //check this with tracing also we could use Ok(event) to get the error
			// If it blocks the main thread too many time we have to change this
		});

		Ok(())
	}

	/// Detach from an active buffer, stopping its underlying worker
	/// this method returns true if no reference or last reference was held, false if there are still
	/// dangling references to clear
	#[napi(js_name = "detachBuffer")]
	pub async fn js_detach_buffer(&self, path: String) -> bool {
		self.detach_buffer(&path)
	}

	/// Re-fetch remote buffer list
	#[napi(js_name = "fetchBuffers")]
	pub async fn js_fetch_buffers(&self) -> napi::Result<()> {
		Ok(self.fetch_buffers().await?)
	}
	/// Re-fetch the list of all users in the workspace.
	#[napi(js_name = "fetchUsers")]
	pub async fn js_fetch_users(&self) -> napi::Result<()> {
		Ok(self.fetch_users().await?)
	}

	/// List users attached to a specific buffer
	#[napi(js_name = "fetchBufferUsers")]
	pub async fn js_fetch_buffer_users(
		&self,
		path: String,
	) -> napi::Result<()> {
		Ok(self.fetch_buffer_users(&path).await?)
	}

	/// Get all users currently attached to specified buffer
	#[napi(js_name = "bufferUserList")]
	pub fn js_buffer_user_list(&self, path: String) -> Vec<CodempUserInfo> {
		self.buffer_user_list(&path)
	}

	/// Pin an ephemeral buffer, making it permanent.
	#[napi(js_name = "pinBuffer")]
	pub async fn js_pin_buffer(&self, path: String) -> napi::Result<()> {
		Ok(self.pin_buffer(&path).await?)
	}

	/// Unpins a permanent buffer, making it ephemeral.
	#[napi(js_name = "unpinBuffer")]
	pub async fn js_un_pin_buffer(&self, path: String) -> napi::Result<()> {
		Ok(self.un_pin_buffer(&path).await?)
	}
}
