use codemp_proto::{cursor::{CursorEvent, CursorUpdate}, session::WorkspaceIdentifier};
use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi_derive::napi;

use crate::{api::{AsyncReceiver, AsyncSender}, cursor::controller::CursorController};

#[napi]
impl CursorController {
	/// Register a callback to be called on receive.
	/// There can only be one callback registered at any given time.
	#[napi(
		js_name = "callback",
		ts_args_type = "fun: (err: Error|null, event: CursorController) => void"
	)]
	pub fn js_callback(
		&self,
		fun: ThreadsafeFunction<CursorController>,
	) -> napi::Result<()> {
		self.callback(move |controller: CursorController| {
			fun.call(Ok(controller.clone()), ThreadsafeFunctionCallMode::Blocking);
			//check this with tracing also we could use Ok(event) to get the error
			// If it blocks the main thread too many time we have to change this
		});

		Ok(())
	}

	/// Clear the registered callback
	#[napi(js_name = "clearCallback")]
	pub fn js_clear_callback(&self) {
		self.clear_callback();
	}

	/// Send a new cursor event to remote
	#[napi(js_name = "send")]
	pub fn js_send(&self, sel: CursorUpdate) -> napi::Result<()> {
    Ok(self.send(sel)?)
}

	/// Get next cursor event if available without blocking
	#[napi(js_name = "tryRecv")]
	pub async fn js_try_recv(&self) -> napi::Result<Option<CursorEvent>> {
		Ok(self.try_recv().await?)
	}

	/// Block until next
	#[napi(js_name = "recv")]
	pub async fn js_recv(&self) -> napi::Result<CursorEvent> {
		Ok(self.recv().await?)
	}

	/// Get id of workspace containing this controller.
	#[napi(js_name = "workspaceId")]
	pub fn js_workspace_id(&self) -> WorkspaceIdentifier {
		self.workspace_id().clone()
	}
}
