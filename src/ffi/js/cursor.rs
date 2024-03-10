use std::sync::Arc;
use napi_derive::napi;
use uuid::Uuid;
use napi::threadsafe_function::{ThreadsafeFunction, ThreadSafeCallContext, ThreadsafeFunctionCallMode, ErrorStrategy};
use crate::api::Controller;
use crate::ffi::js::JsCodempError;

#[napi]
pub struct JsCursorController(Arc<crate::cursor::Controller>);

impl From::<Arc<crate::cursor::Controller>> for JsCursorController {
	fn from(value: Arc<crate::cursor::Controller>) -> Self {
		JsCursorController(value)
	}
}

#[napi]
impl JsCursorController {

	#[napi(ts_args_type = "fun: (event: JsCursorEvent) => void")]
	pub fn callback(&self, fun: napi::JsFunction) -> napi::Result<()>{ 
		let tsfn : ThreadsafeFunction<codemp_proto::cursor::CursorEvent, ErrorStrategy::Fatal> = 
		fun.create_threadsafe_function(0,
			|ctx : ThreadSafeCallContext<codemp_proto::cursor::CursorEvent>| {
				Ok(vec![JsCursorEvent::from(ctx.value)])
			}
		)?;
		let _controller = self.0.clone();
		tokio::spawn(async move {
			loop {
				match _controller.recv().await {
					Ok(event) => {
						tsfn.call(event.clone(), ThreadsafeFunctionCallMode::NonBlocking); //check this shit with tracing also we could use Ok(event) to get the error
					},
					Err(crate::Error::Deadlocked) => continue,
					Err(e) => break tracing::warn!("error receiving: {}", e),
				}
			}
		});
		Ok(())
	}

	#[napi]
	pub fn send(&self, buffer: String, start: (i32, i32), end: (i32, i32)) -> napi::Result<()> {
		let pos = codemp_proto::cursor::CursorPosition {
			buffer: buffer.into(),
			start: codemp_proto::cursor::RowCol::from(start),
			end: codemp_proto::cursor::RowCol::from(end),
		};
		Ok(self.0.send(pos).map_err(JsCodempError)?)
	}
}



#[derive(Debug)]
#[napi(object)]
pub struct JsCursorEvent {
	pub user: String,
	pub buffer: String,
	pub start: JsRowCol,
	pub end: JsRowCol,
}

impl From::<codemp_proto::cursor::CursorEvent> for JsCursorEvent {
	fn from(value: codemp_proto::cursor::CursorEvent) -> Self {
		let pos = value.position;
		let start = pos.start;
		let end = pos.end;
		JsCursorEvent {
			user: Uuid::from(value.user).to_string(),
			buffer: pos.buffer.into(),
			start: JsRowCol { row: start.row, col: start.col },
			end: JsRowCol { row: end.row, col: end.col },
		}
	}
}

#[derive(Debug)]
#[napi(object)]
pub struct JsRowCol {
	pub row: i32,
	pub col: i32
}

impl From::<codemp_proto::cursor::RowCol> for JsRowCol {
	fn from(value: codemp_proto::cursor::RowCol) -> Self {
		JsRowCol { row: value.row, col: value.col }
	}
}