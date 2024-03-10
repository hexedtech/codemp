use std::sync::Arc;
use napi::threadsafe_function::{ErrorStrategy::Fatal, ThreadSafeCallContext, ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi_derive::napi;
use crate::api::Controller;
use crate::ffi::js::JsCodempError;

/// BUFFER
#[napi(object)]
pub struct JsTextChange {
	pub span: JsRange,
	pub content: String,
}

#[napi(object)]
pub struct JsRange{
	pub start: i32,
	pub end: i32,
}

impl From::<crate::api::TextChange> for JsTextChange {
	fn from(value: crate::api::TextChange) -> Self {
		JsTextChange {
			// TODO how is x.. represented ? span.end can never be None
			span: JsRange { start: value.span.start as i32, end: value.span.end as i32 },
			content: value.content,
		}
	}
}


impl From::<Arc<crate::buffer::Controller>> for JsBufferController {
	fn from(value: Arc<crate::buffer::Controller>) -> Self {
		JsBufferController(value)
	}
}


#[napi]
pub struct JsBufferController(Arc<crate::buffer::Controller>);


/*#[napi]
pub fn delta(string : String, start: i64, txt: String, end: i64 ) -> Option<JsCodempOperationSeq> {
	Some(JsCodempOperationSeq(string.diff(start as usize, &txt, end as usize)?))
}*/






#[napi]
impl JsBufferController {


	#[napi(ts_args_type = "fun: (event: JsTextChange) => void")]
	pub fn callback(&self, fun: napi::JsFunction) -> napi::Result<()>{ 
		let tsfn : ThreadsafeFunction<crate::api::TextChange, Fatal> = 
		fun.create_threadsafe_function(0,
			|ctx : ThreadSafeCallContext<crate::api::TextChange>| {
				Ok(vec![JsTextChange::from(ctx.value)])
			}
		)?;
		let _controller = self.0.clone();
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


	#[napi]
	pub fn content(&self) -> String {
		self.0.content()
	}

	#[napi]
	pub fn get_name(&self) -> String {
		self.0.name().to_string()
	}

	#[napi]
	pub async fn recv(&self) -> napi::Result<JsTextChange> {
		Ok(
			self.0.recv().await
				.map_err(|e| napi::Error::from(JsCodempError(e)))?
				.into()
		)
	}

	#[napi]
	pub fn send(&self, op: JsTextChange) -> napi::Result<()> {
		// TODO might be nice to take ownership of the opseq
		let new_text_change = crate::api::TextChange {
			span: op.span.start as usize .. op.span.end as usize,
			content: op.content,
		};
		Ok(self.0.send(new_text_change).map_err(JsCodempError)?)
	}
}