//! ### javascript
//! Using [napi] it's possible to map perfectly the entirety of `codemp` API.
//! Async operations run on a dedicated [tokio] runtime and the result is sent back to main thread

pub mod client;
pub mod workspace;
pub mod cursor;
pub mod buffer;
pub mod op_cache;
pub mod ext;


impl From<crate::errors::ConnectionError> for napi::Error {
	fn from(value: crate::errors::ConnectionError) -> Self {
		napi::Error::new(napi::Status::GenericFailure, format!("{value}"))
	}
}

impl From<crate::errors::RemoteError> for napi::Error {
	fn from(value: crate::errors::RemoteError) -> Self {
		napi::Error::new(napi::Status::GenericFailure, format!("{value}"))
	}
}

impl From<crate::errors::ControllerError> for napi::Error {
	fn from(value: crate::errors::ControllerError) -> Self {
		napi::Error::new(napi::Status::GenericFailure, format!("{value}"))
	}
}

use napi_derive::napi;

#[napi]
pub struct JsLogger(std::sync::Arc<tokio::sync::Mutex<tokio::sync::mpsc::Receiver<String>>>);

#[napi]
impl JsLogger {
	#[napi(constructor)]
	pub fn new(debug: Option<bool>) -> JsLogger {
		let (tx, rx) = tokio::sync::mpsc::channel(256);
		let level = if debug.unwrap_or(false) { tracing::Level::DEBUG } else {tracing::Level::INFO }; //TODO: study this tracing subscriber and customize it
		let format = tracing_subscriber::fmt::format()
			.with_level(true)
			.with_target(true)
			.with_thread_ids(false)
			.with_thread_names(false)
			.with_ansi(false)
			.with_file(false)
			.with_line_number(false)
			.with_source_location(false)
			.compact();
		let _initialized = tracing_subscriber::fmt()
			.event_format(format)
			.with_max_level(level)
			.with_writer(std::sync::Mutex::new(JsLoggerProducer(tx)))
			.try_init()
			.is_ok();
		JsLogger(std::sync::Arc::new(tokio::sync::Mutex::new(rx)))
	}

	#[napi]
	pub async fn message(&self) -> Option<String> {
		self.0
			.lock()
			.await
			.recv()
			.await
	}
}

#[derive(Debug, Clone)]
struct JsLoggerProducer(tokio::sync::mpsc::Sender<String>);

impl std::io::Write for JsLoggerProducer {
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		// TODO this is a LOSSY logger!!
		let _ = self.0.try_send(String::from_utf8_lossy(buf).to_string()); // ignore: logger disconnected or with full buffer
		Ok(buf.len())
	}

	fn flush(&mut self) -> std::io::Result<()> { Ok(()) }
}
