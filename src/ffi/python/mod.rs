mod client;
mod controllers;
mod workspace;

use crate::prelude::*;

use pyo3::{
	exceptions::{PyConnectionError, PyRuntimeError, PySystemError},
	prelude::*,
	types::PyDict,
};

use std::sync::OnceLock;
use tokio::sync::{mpsc, oneshot};

/// global reference to a current_thread tokio runtime
pub fn tokio() -> &'static tokio::runtime::Runtime {
	static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
	RT.get_or_init(|| {
		tokio::runtime::Builder::new_current_thread()
			.enable_all()
			.build()
			.expect("Failed to start the tokio runtime!")
	})
}

// #[pyfunction]
// fn register_event_loop(event_loop: Py<PyAny>) {
// 	static EVENT_LOOP: OnceLock<Py<PyAny>> = OnceLock::new();
// 	EVENT_LOOP.
// }

// #[pyfunction]
// fn setup_async(
// 	event_loop: Py<PyAny>,
// 	call_soon_thread_safe: Py<PyAny>, // asyncio.EventLoop.call_soon_threadsafe
// 	call_coroutine_thread_safe: Py<PyAny>, // asyncio.call_coroutine_threadsafe
// 	create_future: Py<PyAny>,         // asyncio.EventLoop.create_future
// ) {
// 	let _ = EVENT_LOOP.get_or_init(|| event_loop);
// 	let _ = CALL_SOON.get_or_init(|| call_soon_thread_safe);
// 	let _ = CREATE_TASK.get_or_init(|| call_coroutine_thread_safe);
// 	let _ = CREATE_FUTURE.get_or_init(|| create_future);
// }

/// Implements a simple future like object between python and rust to allow for async operations
/// between the two runtimes.
#[pyclass] // once we go past pytho 3.8 we can use pyclass(generic)
pub struct Promise(Option<tokio::task::JoinHandle<PyResult<Py<PyAny>>>>);

#[pymethods]
impl Promise {
	// Can't use this in callbacks since tokio will complain about running
	// a runtime inside another runtime.
	#[pyo3(name = "wait")]
	fn _await(&mut self, py: Python<'_>) -> PyResult<Py<PyAny>> {
		py.detach(move || match self.0.take() {
			None => Err(PyRuntimeError::new_err(
				"promise can't be awaited multiple times!",
			)),
			Some(x) => match tokio().block_on(x) {
				Err(e) => Err(PyRuntimeError::new_err(format!(
					"error awaiting promise: {e}"
				))),
				Ok(res) => res,
			},
		})
	}

	fn done(&self, py: Python<'_>) -> PyResult<bool> {
		py.detach(|| {
			if let Some(handle) = &self.0 {
				Ok(handle.is_finished())
			} else {
				Err(PyRuntimeError::new_err("promise was already awaited."))
			}
		})
	}
}

// macro_rules! a_sync {
// 	($x:expr) => {{
// 		Ok($crate::ffi::python::Promise(Some(
// 			$crate::ffi::python::tokio().spawn(async move {
// 				let res = $x?;
// 				Python::attach(|py| Ok(res.into_pyobject(py)?.into_any().unbind()))
// 			}),
// 		)))
// 	}};
// }
//pub(crate) use a_sync;

/// creates a future that will run detached from the gil, up until when it will need to resolve itself.
/// at which point it will wait for the gil to become available, attach itself and populate the promise
macro_rules! a_sync_detach {
	($py:ident, $x:expr) => {{
		$py.detach(move || {
			Ok($crate::ffi::python::Promise(Some(
				$crate::ffi::python::tokio().spawn(async move {
					let res = $x?;
					Python::attach(|gil| Ok(res.into_pyobject(gil)?.into_any().unbind()))
				}),
			)))
		})
	}};
}
pub(crate) use a_sync_detach;

#[derive(Debug, Clone)]
struct LoggerProducer(mpsc::UnboundedSender<String>);

impl std::io::Write for LoggerProducer {
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		let _ = self.0.send(String::from_utf8_lossy(buf).to_string());
		Ok(buf.len())
	}

	fn flush(&mut self) -> std::io::Result<()> {
		Ok(())
	}
}

/// This is a helper struct that allows for managing (i.e. stop the rust tokio runtime) from python directly.
#[pyclass]
pub struct Driver(Option<oneshot::Sender<()>>);
#[pymethods]
impl Driver {
	fn stop(&mut self) -> PyResult<()> {
		match self.0.take() {
			Some(tx) => {
				let _ = tx.send(());
				Ok(())
			}
			None => Err(PySystemError::new_err("Runtime was already stopped.")),
		}
	}
}

#[pyfunction]
fn version() -> &'static str {
	crate::version()
}

#[pyfunction]
fn init() -> PyResult<Driver> {
	let (rt_stop_tx, mut rt_stop_rx) = oneshot::channel::<()>();
	std::thread::spawn(move || {
		tokio().block_on(async move {
			tracing::info!("started runtime driver...");
			tokio::select! {
				() = std::future::pending::<()>() => {},
				_ = &mut rt_stop_rx => {}
			}
		})
	});

	Ok(Driver(Some(rt_stop_tx)))
}

// #[pymethods]
// impl CodempUserInfo {
// 	#[getter]
// 	fn get_name(&self) -> pyo3::PyResult<String> {
// 		Ok(self.name.clone())
// 	}
//
// 	#[getter]
// 	fn get_display_name(&self) -> pyo3::PyResult<String> {
// 		Ok(self.display_name.clone().unwrap_or(self.name.clone()))
// 	}
//
// 	#[setter]
// 	fn set_display_name(&mut self, value: String) -> pyo3::PyResult<()> {
// 		self.display_name.replace(value);
//
// 		Ok(())
// 	}
//
// 	#[getter]
// 	fn get_description(&self) -> pyo3::PyResult<String> {
// 		Ok(self
// 			.description
// 			.clone()
// 			.unwrap_or("No description.".to_string()))
// 	}
//
// 	#[setter]
// 	fn set_description(&mut self, value: String) -> pyo3::PyResult<()> {
// 		self.description.replace(value);
// 		Ok(())
// 	}
//
// 	fn __str__(&self) -> String {
// 		format!("{self:?}")
// 	}
// }

#[pymethods]
impl CodempConfig {
	/// Creates a new config, handling defaulted values.
	#[new]
	#[pyo3(signature = (*, username, password, **kwds))]
	pub fn pynew(
		username: String,
		password: String,
		kwds: Option<Bound<'_, PyDict>>,
	) -> PyResult<Self> {
		if let Some(kwgs) = kwds {
			let host = kwgs.get_item("host")?.and_then(|e| e.extract().ok());
			let port = kwgs.get_item("port")?.and_then(|e| e.extract().ok());
			let tls = kwgs.get_item("tls")?.and_then(|e| e.extract().ok());

			Ok(Self {
				username,
				password,
				host,
				port,
				tls,
			})
		} else {
			Ok(Self::new(username, password))
		}
	}

	fn __str__(&self) -> String {
		format!("{self:?}")
	}
}

// #[pymethods]
// impl CodempCursor {
// 	fn __str__(&self) -> String {
// 		format!("{self:?}")
// 	}
// }

// #[pymethods]
// impl CodempCursorPosition {
// 	#[new]
// 	#[pyo3(signature = (*, start_row, start_col, end_row, end_col, **kwds))]
// 	pub fn py_new(
// 		start_row: i32,
// 		start_col: i32,
// 		end_row: i32,
// 		end_col: i32,
// 		kwds: Option<&Bound<'_, PyDict>>,
// 	) -> PyResult<Self> {
// 		if let Some(_kwds) = kwds {
// 			Ok(Self {
// 				start_row,
// 				start_col,
// 				end_row,
// 				end_col,
// 			})
// 		} else {
// 			Ok(Self {
// 				start_row,
// 				start_col,
// 				end_row,
// 				end_col,
// 			})
// 		}
// 	}
//
// 	fn __str__(&self) -> String {
// 		format!("{self:?}")
// 	}
// }

// #[pymethods]
// impl CodempBufferUpdate {
// 	fn __str__(&self) -> String {
// 		format!("{self:?}")
// 	}
// }

// #[pymethods]
// impl CodempTextChange {
// 	#[new]
// 	#[pyo3(signature = (*, start, end, content, **kwds))]
// 	pub fn py_new(
// 		start: u32,
// 		end: u32,
// 		content: String,
// 		kwds: Option<&Bound<'_, PyDict>>,
// 	) -> PyResult<Self> {
// 		if let Some(_kwds) = kwds {
// 			Ok(Self {
// 				start_idx: start,
// 				end_idx: end,
// 				content,
// 			})
// 		} else {
// 			Ok(Self {
// 				start_idx: start,
// 				end_idx: end,
// 				content,
// 			})
// 		}
// 	}
//
// 	fn __str__(&self) -> String {
// 		format!("{self:?}")
// 	}
// }

#[pyfunction]
fn connect(py: Python, config: Py<CodempConfig>) -> PyResult<Promise> {
	let conf: CodempConfig = config.extract(py)?;
	Ok(Promise(Some(crate::ffi::python::tokio().spawn(
		async move {
			let client = CodempClient::connect(conf).await?;
			Python::attach(|py| Ok(client.into_pyobject(py)?.into_any().unbind()))
		},
	))))
	// a_sync!(Client::connect(conf).await)
}

#[pyfunction]
fn set_logger(py: Python, logging_cb: Py<PyAny>, debug: bool) -> bool {
	if !logging_cb.bind_borrowed(py).is_callable() {
		return false;
	}
	let (tx, mut rx) = mpsc::unbounded_channel();
	let level = if debug {
		tracing::Level::DEBUG
	} else {
		tracing::Level::INFO
	};

	let format = tracing_subscriber::fmt::format()
		.without_time()
		.with_level(true)
		.with_target(true)
		.with_thread_ids(false)
		.with_thread_names(true)
		.with_file(false)
		.with_line_number(false)
		.with_source_location(true)
		.compact();

	let log_subscribed = tracing_subscriber::fmt()
		.with_ansi(false)
		.event_format(format)
		.with_max_level(level)
		.with_writer(std::sync::Mutex::new(LoggerProducer(tx)))
		.try_init()
		.is_ok();

	if log_subscribed {
		tokio().spawn(async move {
			while let Some(msg) = rx.recv().await {
				let _ = Python::attach(|py| logging_cb.call1(py, (msg,)));
			}
		});
	}
	log_subscribed
}

impl From<crate::errors::ConnectionError> for PyErr {
	fn from(value: crate::errors::ConnectionError) -> Self {
		PyConnectionError::new_err(format!("Connection error: {value}"))
	}
}

impl From<crate::errors::RemoteError> for PyErr {
	fn from(value: crate::errors::RemoteError) -> Self {
		PyRuntimeError::new_err(format!("Remote error: {value}"))
	}
}

impl From<crate::errors::ControllerError> for PyErr {
	fn from(value: crate::errors::ControllerError) -> Self {
		PyRuntimeError::new_err(format!("Controller error: {value}"))
	}
}

// #[pymodule]
// fn codemp(m: &Bound<'_, PyModule>) -> PyResult<()> {
// 	m.add_function(wrap_pyfunction!(version, m)?)?;
// 	m.add_function(wrap_pyfunction!(init, m)?)?;
// 	m.add_function(wrap_pyfunction!(connect, m)?)?;
// 	m.add_function(wrap_pyfunction!(set_logger, m)?)?;
// 	m.add_class::<Driver>()?;

// 	m.add_class::<CodempBufferUpdate>()?;
// 	m.add_class::<CodempTextChange>()?;
// 	m.add_class::<CodempBufferController>()?;

// 	m.add_class::<CodempCursorUpdate>()?;
// 	m.add_class::<CodempCursorPosition>()?;
// 	m.add_class::<CodempCursorController>()?;

// 	m.add_class::<CodempUserInfo>()?;

// 	m.add_class::<CodempWorkspace>()?;
// 	m.add_class::<CodempWorkspaceEvent>()?;
// 	m.add_class::<CodempSessionEvent>()?;
// 	m.add_class::<CodempClient>()?;
// 	m.add_class::<CodempConfig>()?;

// 	Ok(())
// }

#[pymodule(name = "codemplib")]
mod pycodemp {
	#[pymodule_export]
	use super::version;

	#[pymodule_export]
	use super::init;

	#[pymodule_export]
	use super::connect;

	#[pymodule_export]
	use super::set_logger;

	#[pymodule_export]
	use super::Driver;

	#[pymodule_export]
	use super::Promise;

	#[pymodule_export]
	use super::CodempBufferController;

	#[pymodule_export]
	use super::CodempBufferUpdate;

	#[pymodule_export]
	use super::CodempTextChange;

	#[pymodule_export]
	use super::CodempCursorPosition;

	#[pymodule_export]
	use super::CodempCursorUpdate;

	#[pymodule_export]
	use super::CodempCursorController;

	#[pymodule_export]
	use super::CodempUserInfo;

	#[pymodule_export]
	use super::CodempWorkspace;

	#[pymodule_export]
	use super::CodempWorkspaceIdentifier;

	#[pymodule_export]
	use super::CodempWorkspaceEvent;

	#[pymodule_export]
	use super::CodempSessionEvent;

	#[pymodule_export]
	use super::CodempClient;

	#[pymodule_export]
	use super::CodempConfig;
}
