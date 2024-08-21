use napi::threadsafe_function::{ErrorStrategy::Fatal, ThreadSafeCallContext, ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi_derive::napi;
use crate::api::TextChange;
use crate::api::Controller;
use crate::buffer::controller::BufferController;


#[napi]
impl BufferController {
	#[napi(js_name = "callback", ts_args_type = "fun: (event: TextChange) => void")]
	pub fn jscallback(&self, fun: napi::JsFunction) -> napi::Result<()>{ 
		let tsfn : ThreadsafeFunction<crate::api::TextChange, Fatal> = 
		fun.create_threadsafe_function(0,
			|ctx : ThreadSafeCallContext<crate::api::TextChange>| {
				Ok(vec![ctx.value])
			}
		)?;
		let _controller = self.clone();
		tokio::spawn(async move {
			//tokio::time::sleep(std::time::Duration::from_secs(1)).await;
			loop {
				tokio::time::sleep(std::time::Duration::from_millis(200)).await;
				match _controller.recv().await {
					Ok(event) => {
						tsfn.call(event, ThreadsafeFunctionCallMode::NonBlocking); //check this with tracing also we could use Ok(event) to get the error
					},
					Err(crate::Error::Deadlocked) => continue,
					Err(e) => break tracing::warn!("error receiving: {}", e),
				}
			}
		});
		Ok(())
	}

	#[napi(js_name = "try_recv")]
	pub async fn js_try_recv(&self) -> napi::Result<Option<TextChange>> {
		Ok(self.try_recv().await?)
	}

	#[napi(js_name = "recv")]
	pub async fn js_recv(&self) -> napi::Result<TextChange> {
		Ok(self.recv().await?.into())
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
