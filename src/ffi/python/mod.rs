pub mod client;
pub mod controllers;
pub mod workspace;

use crate::{
	api::{Cursor, TextChange},
	buffer::Controller as BufferController,
	cursor::Controller as CursorController,
	Client, Workspace,
};

use pyo3::prelude::*;
use pyo3::{
	exceptions::{PyConnectionError, PyRuntimeError, PySystemError},
	types::PyFunction,
};

use tokio::sync::{mpsc, oneshot};

// global reference to a current_thread tokio runtime
pub fn tokio() -> &'static tokio::runtime::Runtime {
	use std::sync::OnceLock;
	static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
	RT.get_or_init(|| {
		tokio::runtime::Builder::new_current_thread()
			.enable_all()
			.on_thread_start(|| tracing::info!("tokio thread started."))
			.on_thread_stop(|| tracing::info!("tokio thread stopped."))
			.build()
			.unwrap()
	})
}

#[pyclass]
pub struct Promise(Option<tokio::task::JoinHandle<PyResult<PyObject>>>);

#[pymethods]
impl Promise {
	#[pyo3(name = "wait")]
	fn _await(&mut self, py: Python<'_>) -> PyResult<PyObject> {
		py.allow_threads(move || match self.0.take() {
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
		py.allow_threads(|| {
			if let Some(handle) = &self.0 {
				Ok(handle.is_finished())
			} else {
				Err(PyRuntimeError::new_err("promise was already awaited."))
			}
		})
	}
}

#[macro_export]
macro_rules! a_sync {
	($x:expr) => {{
		Ok($crate::ffi::python::Promise(Some(
			$crate::ffi::python::tokio()
				.spawn(async move { Ok($x.map(|f| Python::with_gil(|py| f.into_py(py)))?) }),
		)))
	}};
}

#[macro_export]
macro_rules! a_sync_allow_threads {
	($py:ident, $x:expr) => {{
		$py.allow_threads(move || {
			Ok($crate::ffi::python::Promise(Some(
				$crate::ffi::python::tokio()
					.spawn(async move { Ok($x.map(|f| Python::with_gil(|py| f.into_py(py)))?) }),
			)))
		})
	}};
}

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
fn init(logging_cb: Py<PyFunction>, debug: bool) -> PyResult<PyObject> {
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
		.with_thread_names(false)
		.with_file(false)
		.with_line_number(false)
		.with_source_location(false)
		.compact();

	let log_subscribing = tracing_subscriber::fmt()
		.with_ansi(false)
		.event_format(format)
		.with_max_level(level)
		.with_writer(std::sync::Mutex::new(LoggerProducer(tx)))
		.try_init();

	let (rt_stop_tx, mut rt_stop_rx) = oneshot::channel::<()>();

	match log_subscribing {
		Ok(_) => {
			// the runtime is driven by the logger awaiting messages from codemp and echoing them back to
			// python logger.
			std::thread::spawn(move || {
				tokio().block_on(async move {
					loop {
						tokio::select! {
							biased;
							Some(msg) = rx.recv() => {
								let _ = Python::with_gil(|py| logging_cb.call1(py, (msg,)));
							},
							_ = &mut rt_stop_rx => { todo!() },
						}
					}
				})
			});
			Ok(Python::with_gil(|py| Driver(Some(rt_stop_tx)).into_py(py)))
		}
		Err(_) => Err(PyRuntimeError::new_err("codemp was already initialised.")),
	}
}

impl From<crate::Error> for PyErr {
	fn from(value: crate::Error) -> Self {
		match value {
			crate::Error::Transport { status, message } => {
				PyConnectionError::new_err(format!("Transport error: ({}) {}", status, message))
			}
			crate::Error::Channel { send } => {
				PyConnectionError::new_err(format!("Channel error (send:{})", send))
			}
			crate::Error::InvalidState { msg } => {
				PyRuntimeError::new_err(format!("Invalid state: {}", msg))
			}
			crate::Error::Deadlocked => PyRuntimeError::new_err("Deadlock, retry."),
		}
	}
}

impl IntoPy<PyObject> for crate::api::User {
	fn into_py(self, py: Python<'_>) -> PyObject {
		self.id.to_string().into_py(py)
	}
}

#[pymodule]
fn codemp(m: &Bound<'_, PyModule>) -> PyResult<()> {
	m.add_function(wrap_pyfunction!(init, m)?)?;
	m.add_class::<Driver>()?;

	m.add_class::<TextChange>()?;
	m.add_class::<BufferController>()?;

	m.add_class::<Cursor>()?;
	m.add_class::<CursorController>()?;

	m.add_class::<Workspace>()?;
	m.add_class::<Client>()?;

	Ok(())
}
