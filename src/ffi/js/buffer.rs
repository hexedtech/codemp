use crate::api::change::Delta;
use crate::api::controller::{AsyncReceiver, AsyncSender};
use crate::api::TextChange;
use crate::buffer::controller::BufferController;
use napi::threadsafe_function::{
	ErrorStrategy::Fatal, ThreadSafeCallContext, ThreadsafeFunction, ThreadsafeFunctionCallMode,
};
use napi::threadsafe_function::{
	ErrorStrategy::Fatal, ThreadSafeCallContext, ThreadsafeFunction, ThreadsafeFunctionCallMode,
};
use napi_derive::napi;
use napi_derive::napi;

#[napi]
impl BufferController {
	/// Register a callback to be invoked every time a new event is available to consume
	/// There can only be one callback registered at any given time.
	#[napi(
		js_name = "callback",
		ts_args_type = "fun: (event: BufferController) => void"
	)]
	pub fn js_callback(&self, fun: napi::JsFunction) -> napi::Result<()> {
		let tsfn: ThreadsafeFunction<crate::buffer::controller::BufferController, Fatal> = fun
			.create_threadsafe_function(
				0,
				|ctx: ThreadSafeCallContext<crate::buffer::controller::BufferController>| {
					Ok(vec![ctx.value])
				},
			)?;
		self.callback(move |controller: BufferController| {
			tsfn.call(controller.clone(), ThreadsafeFunctionCallMode::Blocking);
			//check this with tracing also we could use Ok(event) to get the error
			// If it blocks the main thread too many time we have to change this
		});

		Ok(())
	}

	/// Remove registered buffer callback
	#[napi(js_name = "clear_callback")]
	pub fn js_clear_callback(&self) {
		self.clear_callback();
	}

	/// Get buffer path
	#[napi(js_name = "get_path")]
	pub fn js_path(&self) -> &str {
		self.path()
	}

	/// Block until next buffer event without returning it
	#[napi(js_name = "poll")]
	pub async fn js_poll(&self) -> napi::Result<()> {
		Ok(self.poll().await?)
	}

	/// Return next buffer event if present
	#[napi(js_name = "try_recv")]
	pub async fn js_try_recv(&self) -> napi::Result<Option<Delta<BufferAck>>> {
		Ok(self.try_recv().await?)
	}

	/// Wait for next buffer event and return it
	#[napi(js_name = "recv")]
	pub async fn js_recv(&self) -> napi::Result<Delta<BufferAck>> {
		Ok(self.recv().await?)
	}

	/// Send a buffer update to workspace
	#[napi(js_name = "send")]
	pub fn js_send(&self, op: TextChange) -> napi::Result<()> {
		Ok(self.send(op)?)
	}

	/// Return buffer whole content
	#[napi(js_name = "content")]
	pub async fn js_content(&self) -> napi::Result<String> {
		Ok(self.content().await?)
	}
}
