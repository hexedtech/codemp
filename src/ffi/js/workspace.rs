use napi::threadsafe_function::ErrorStrategy::Fatal;
use napi::threadsafe_function::{ThreadSafeCallContext, ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi_derive::napi;
use crate::Workspace;
use crate::buffer::controller::BufferController;
use crate::cursor::controller::CursorController;
use crate::api::controller::AsyncReceiver;

#[napi(object, js_name = "Event")]
pub struct JsEvent {
	pub r#type: String,
	pub value: String,
}

impl From<crate::api::Event> for JsEvent {
	fn from(value: crate::api::Event) -> Self {
		match value {
			crate::api::Event::FileTreeUpdated(value) => Self { r#type: "filetree".into(), value },
			crate::api::Event::UserJoin(value) => Self { r#type: "join".into(), value },
			crate::api::Event::UserLeave(value) => Self { r#type: "leave".into(), value },
		}
	}
}

#[napi]
impl Workspace {
	#[napi(js_name = "id")]
	pub fn js_id(&self) -> String {
		self.id()
	}
	
	#[napi(js_name = "filetree")]
	pub fn js_filetree(&self, filter: Option<&str>, strict: bool) -> Vec<String> {
		self.filetree(filter, strict)
	}

	#[napi(js_name = "user_list")]
	pub fn js_user_list(&self) -> Vec<String> {
		self.user_list()
	}

	#[napi(js_name = "cursor")]
	pub fn js_cursor(&self) -> CursorController {
		self.cursor()
	}

	#[napi(js_name = "buffer_by_name")]
	pub fn js_buffer_by_name(&self, path: String) -> Option<BufferController> {
		self.buffer_by_name(&path)
	}

	#[napi(js_name = "create")]
	pub async fn js_create(&self, path: String) -> napi::Result<()> {
		Ok(self.create(&path).await?)
	}

	#[napi(js_name = "attach")]
	pub async fn js_attach(&self, path: String) -> napi::Result<BufferController> {
		Ok(self.attach(&path).await?)
	}
	
	#[napi(js_name = "delete")]
	pub async fn js_delete(&self, path: String) -> napi::Result<()> {
		Ok(self.delete(&path).await?)
	}

	#[napi(js_name = "recv")]
	pub async fn js_recv(&self) -> napi::Result<JsEvent> {
		Ok(JsEvent::from(self.recv().await?))
	}

	#[napi(js_name = "try_recv")]
	pub async fn js_try_recv(&self) -> napi::Result<Option<JsEvent>> {
		Ok(self.try_recv().await?.map(JsEvent::from))
	}

	#[napi(js_name = "poll")]
	pub async fn js_poll(&self) -> napi::Result<()> {
		self.poll().await?;
		Ok(())
	}

	#[napi(js_name = "clear_callback")]
	pub fn js_clear_callbacl(&self) -> napi::Result<()> {
		self.clear_callback();
		Ok(())
	}

	#[napi(js_name = "callback", ts_args_type = "fun: (event: Workspace) => void")]
	pub fn js_callback(&self, fun: napi::JsFunction) -> napi::Result<()>{
		let tsfn : ThreadsafeFunction<crate::Workspace, Fatal> = 
		fun.create_threadsafe_function(0,
			|ctx : ThreadSafeCallContext<crate::Workspace>| {
				Ok(vec![ctx.value])
			}
		)?;
		self.callback(move |controller : Workspace| {

			tsfn.call(controller.clone(), ThreadsafeFunctionCallMode::Blocking); //check this with tracing also we could use Ok(event) to get the error
			// If it blocks the main thread too many time we have to change this

		});

		Ok(())
	}
}
