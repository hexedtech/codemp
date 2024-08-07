use std::sync::Arc;
use napi_derive::napi;
use uuid::Uuid;
use napi::threadsafe_function::{ThreadsafeFunction, ThreadSafeCallContext, ThreadsafeFunctionCallMode, ErrorStrategy};
use crate::api::Controller;
use crate::prelude::*;




#[napi(js_name = "Cursor")]
pub struct JsCursor {
	/// range of text change, as char indexes in buffer previous state
	pub start_row: i32,
	pub start_col: i32,
	pub end_row: i32,
	pub end_col: i32,
	pub buffer: String,
	pub user: Option<String>,
}

impl From<JsCursor> for CodempCursor {
	fn from(value: JsCursor) -> Self {
		CodempCursor {
			start : (value.start_row, value.start_col),
			end:  (value.end_row, value.end_col),
			buffer: value.buffer,
			user: value.user.map(|x| uuid::Uuid::parse_str(&x).expect("invalid uuid")),
		}
	}
}
impl From<CodempCursor> for JsCursor {
	fn from(value: CodempCursor) -> Self {
		JsCursor {
			start_row : value.start.0,
			start_col : value.start.1,
			end_row : value.end.0,
			end_col: value.end.1,
			buffer: value.buffer,
			user: value.user.map(|x| x.to_string())
		}
		
	}
}


#[napi]
impl CodempCursorController {

	#[napi(ts_args_type = "fun: (event: JsCursorEvent) => void")]
	pub fn callback(&self, fun: napi::JsFunction) -> napi::Result<()>{ 
		let tsfn : ThreadsafeFunction<JsCursor, ErrorStrategy::Fatal> = 
		fun.create_threadsafe_function(0,
			|ctx : ThreadSafeCallContext<JsCursor>| {
				Ok(vec![ctx.value])
			}
		)?;
		let _controller = self.clone();
		tokio::spawn(async move {
			loop {
				match _controller.recv().await {
					Ok(event) => {
						tsfn.call(event.into(), ThreadsafeFunctionCallMode::NonBlocking); //check this shit with tracing also we could use Ok(event) to get the error
					},
					Err(crate::Error::Deadlocked) => continue,
					Err(e) => break tracing::warn!("error receiving: {}", e),
				}
			}
		});
		Ok(())
	}

	#[napi]
	pub fn send(&self, pos: &CodempCursorController) -> napi::Result<()> {
		Ok(self.send(pos)?)
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