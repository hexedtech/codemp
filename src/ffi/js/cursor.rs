use napi::threadsafe_function::ErrorStrategy::Fatal;
use napi_derive::napi;
use napi::threadsafe_function::{ThreadsafeFunction, ThreadSafeCallContext, ThreadsafeFunctionCallMode};
use crate::api::Controller;
use crate::cursor::controller::CursorController;


#[napi(object, js_name = "Cursor")]
pub struct JsCursor {
	/// range of text change, as char indexes in buffer previous state
	pub start_row: i32,
	pub start_col: i32,
	pub end_row: i32,
	pub end_col: i32,
	pub buffer: String,
	pub user: Option<String>,
}

impl From<JsCursor> for crate::api::Cursor {
	fn from(value: JsCursor) -> Self {
		crate::api::Cursor {
			start : (value.start_row, value.start_col),
			end:  (value.end_row, value.end_col),
			buffer: value.buffer,
			user: value.user,
		}
	}
}
impl From<crate::api::Cursor> for JsCursor {
	fn from(value: crate::api::Cursor) -> Self {
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
impl CursorController {

	#[napi(js_name = "callback", ts_args_type = "fun: (event: CursorController) => void")]
	pub fn js_callback(&self, fun: napi::JsFunction) -> napi::Result<()>{
		let tsfn : ThreadsafeFunction<crate::cursor::controller::CursorController, Fatal> = 
		fun.create_threadsafe_function(0,
			|ctx : ThreadSafeCallContext<crate::cursor::controller::CursorController>| {
				Ok(vec![ctx.value])
			}
		)?;
		self.callback(move |controller : CursorController| {

			tsfn.call(controller.clone(), ThreadsafeFunctionCallMode::Blocking); //check this with tracing also we could use Ok(event) to get the error
			// If it blocks the main thread too many time we have to change this

		});

		Ok(())
	}



	#[napi(js_name = "send")]
	pub async fn js_send(&self, pos: JsCursor) -> napi::Result<()> {
		Ok(self.send(crate::api::Cursor::from(pos)).await?)
	}


	#[napi(js_name= "try_recv")]
	pub async fn js_try_recv(&self) -> napi::Result<Option<JsCursor>> {
		Ok(self.try_recv().await?
		.map(JsCursor::from))
	}

	#[napi(js_name= "recv")]
	pub async fn js_recv(&self) -> napi::Result<JsCursor> {
		Ok(self.recv().await?.into())
	}
}
