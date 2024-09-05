use napi::threadsafe_function::{ErrorStrategy::Fatal, ThreadSafeCallContext, ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi_derive::napi;
use crate::api::TextChange;
use crate::api::Controller;
use crate::buffer::controller::BufferController;


#[napi]
impl BufferController {

	#[napi(js_name = "callback", ts_args_type = "fun: (event: BufferController) => void")]
	pub fn js_callback(&self, fun: napi::JsFunction) -> napi::Result<()>{
		let tsfn : ThreadsafeFunction<crate::buffer::controller::BufferController, Fatal> = 
		fun.create_threadsafe_function(0,
			|ctx : ThreadSafeCallContext<crate::buffer::controller::BufferController>| {
				Ok(vec![ctx.value])
			}
		)?;
		self.callback(move |controller : BufferController| {

			tsfn.call(controller.clone(), ThreadsafeFunctionCallMode::Blocking); //check this with tracing also we could use Ok(event) to get the error
			// If it blocks the main thread too many time we have to change this

		});

		Ok(())
	}

	#[napi(js_name = "clear_callback")]
	pub fn js_clear_callback(&self) -> napi::Result<()> {
		self.clear_callback();
		Ok(())
	}


	#[napi(js_name = "get_path")]
	pub fn js_path(&self) -> napi::Result<&str> {
		Ok(self.path())
	}

	#[napi(js_name = "poll")]
	pub async fn js_poll(&self) -> napi::Result<()>{
		Ok(self.poll().await?)
	}

	#[napi(js_name = "try_recv")]
	pub async fn js_try_recv(&self) -> napi::Result<Option<TextChange>> {
		Ok(self.try_recv().await?)
	}

	#[napi(js_name = "recv")]
	pub async fn js_recv(&self) -> napi::Result<TextChange> {
		Ok(self.recv().await?)
	}

	#[napi(js_name = "send")]
	pub async fn js_send(&self, op: TextChange) -> napi::Result<()> {
		Ok(self.send(op).await?)
	}

	#[napi(js_name = "content")]
	pub async fn js_content(&self) -> napi::Result<String> {
		Ok(self.content().await?)
	}
}
