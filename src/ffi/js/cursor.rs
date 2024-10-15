use crate::api::controller::{AsyncReceiver, AsyncSender};
use crate::cursor::controller::CursorController;
use napi::threadsafe_function::ErrorStrategy::Fatal;
use napi::threadsafe_function::{
	ThreadSafeCallContext, ThreadsafeFunction, ThreadsafeFunctionCallMode,
};
use napi_derive::napi;

#[napi]
impl CursorController {
	/// Register a callback to be called on receive.
	/// There can only be one callback registered at any given time.
	#[napi(
		js_name = "callback",
		ts_args_type = "fun: (event: CursorController) => void"
	)]
	pub fn js_callback(&self, fun: napi::JsFunction) -> napi::Result<()> {
		let tsfn: ThreadsafeFunction<crate::cursor::controller::CursorController, Fatal> = fun
			.create_threadsafe_function(
				0,
				|ctx: ThreadSafeCallContext<crate::cursor::controller::CursorController>| {
					Ok(vec![ctx.value])
				},
			)?;
		self.callback(move |controller: CursorController| {
			tsfn.call(controller.clone(), ThreadsafeFunctionCallMode::Blocking);
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
	pub fn js_send(&self, sel: crate::api::Selection) -> napi::Result<()> {
		Ok(self.send(sel)?)
	}

	/// Get next cursor event if available without blocking
	#[napi(js_name = "tryRecv")]
	pub async fn js_try_recv(&self) -> napi::Result<Option<crate::api::Cursor>> {
		Ok(self.try_recv().await?.map(crate::api::Cursor::from))
	}

	/// Block until next
	#[napi(js_name = "recv")]
	pub async fn js_recv(&self) -> napi::Result<crate::api::Cursor> {
		Ok(self.recv().await?)
	}
}
