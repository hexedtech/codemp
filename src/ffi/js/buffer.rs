use std::sync::Arc;
use napi::threadsafe_function::{ErrorStrategy::Fatal, ThreadSafeCallContext, ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi_derive::napi;
use crate::api::TextChange;
use crate::ffi::js::JsCodempError;
use crate::api::Controller;
use crate::prelude::*;



impl From<crate::Error> for napi::Error {
	fn from(value: crate::Error) -> Self {
		let msg = format!("{value}");
		match value {
			crate::Error::Deadlocked => napi::Error::new(napi::Status::WouldDeadlock, msg),
			_ => napi::Error::new(napi::Status::GenericFailure, msg),
		}
	}
}

#[napi]
impl CodempBufferController {


	#[napi(ts_args_type = "fun: (event: JsTextChange) => void")]
	pub fn callback(&self, fun: napi::JsFunction) -> napi::Result<()>{ 
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
						tsfn.call(event, ThreadsafeFunctionCallMode::NonBlocking); //check this shit with tracing also we could use Ok(event) to get the error
					},
					Err(crate::Error::Deadlocked) => continue,
					Err(e) => break tracing::warn!("error receiving: {}", e),
				}
			}
		});
		Ok(())
	}


	#[napi(js_name = "recv")]
	pub async fn jsrecv(&self) -> napi::Result<TextChange> {
		Ok(self.recv().await?.into())
	}

	#[napi]
	pub fn send(&self, op: TextChange) -> napi::Result<()> {
		Ok(self.send(op)?)
	}
}