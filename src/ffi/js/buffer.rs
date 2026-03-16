use crate::api::controller::{AsyncReceiver, AsyncSender};
use crate::prelude::*;
use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi_derive::napi;

#[napi]
impl CodempBufferController {
	/// Register a callback to be invoked every time a new event is available to consume
	/// There can only be one callback registered at any given time.
	#[napi(
		js_name = "callback",
		ts_args_type = "fun: (event: BufferController) => void"
	)]
	pub fn js_callback(
		&self,
		fun: ThreadsafeFunction<CodempBufferController>,
	) -> napi::Result<()> {
		self.callback(move |controller: CodempBufferController| {
			fun.call(Ok(controller.clone()), ThreadsafeFunctionCallMode::Blocking);
			//check this with tracing also we could use Ok(event) to get the error
			// If it blocks the main thread too many time we have to change this
		});

		Ok(())
	}

	/// Acknowledge TextChange
	#[napi(js_name = "ack")]
	pub fn js_ack(&self, version: Vec<i64>) {
		self.ack(version);
	}

	/// Remove registered buffer callback
	#[napi(js_name = "clearCallback")]
	pub fn js_clear_callback(&self) {
		self.clear_callback();
	}

	/// Get buffer path
	#[napi(js_name = "path")]
	pub fn js_path(&self) -> &str {
		self.path()
	}

	/// Block until next buffer event without returning it
	#[napi(js_name = "poll")]
	pub async fn js_poll(&self) -> napi::Result<()> {
		Ok(self.poll().await?)
	}

	/// Return next buffer event if present
	#[napi(js_name = "tryRecv")]
	pub async fn js_try_recv(&self) -> napi::Result<Option<crate::api::BufferUpdate>> {
		Ok(self.try_recv().await?)
	}

	/// Wait for next buffer event and return it
	#[napi(js_name = "recv")]
	pub async fn js_recv(&self) -> napi::Result<crate::api::BufferUpdate> {
		Ok(self.recv().await?)
	}

	/// Send a buffer update to workspace
	#[napi(js_name = "send")]
	pub fn js_send(&self, op: CodempTextChange) -> napi::Result<()> {
		Ok(self.send(op)?)
	}

	/// Return buffer whole content
	#[napi(js_name = "content")]
	pub async fn js_content(&self) -> napi::Result<String> {
		Ok(self.content().await?)
	}

	/// Get id of workspace containing this controller.
	#[napi(js_name = "workspaceId")]
	pub fn js_workspace_id(&self) -> CodempWorkspaceIdentifier {
		self.workspace_id().clone().into()
	}
}
