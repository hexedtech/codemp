pub mod client;
pub mod controllers;
pub mod workspace;

use std::{
	future::Future,
	pin::Pin,
	sync::OnceLock,
	task::{Context, Poll},
};

use crate::{
	api::{Cursor, TextChange},
	buffer::Controller as BufferController,
	cursor::Controller as CursorController,
	Client, Workspace,
};
use pyo3::prelude::*;
use pyo3::{
	exceptions::{PyConnectionError, PyRuntimeError, PySystemError},
	ffi::PyFunctionObject,
	types::PyFunction,
};
use tokio::sync::watch;

pub fn tokio() -> &'static tokio::runtime::Runtime {
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

// workaround to allow the GIL to be released across awaits, waiting on
// https://github.com/PyO3/pyo3/pull/3610
struct AllowThreads<F>(F);

impl<F> Future for AllowThreads<F>
where
	F: Future + Unpin + Send,
	F::Output: Send,
{
	type Output = F::Output;

	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		let waker = cx.waker();
		let fut = unsafe { self.map_unchecked_mut(|e| &mut e.0) };
		Python::with_gil(|py| py.allow_threads(|| fut.poll(&mut Context::from_waker(waker))))
	}
}

#[macro_export]
macro_rules! a_sync {
	($($clone:ident)* => $x:expr) => {
		{
			$(let $clone = $clone.clone();)*
			Ok(Promise(Some($crate::ffi::python::tokio().spawn(async move { $x }))))
		}
	};
}

#[macro_export]
macro_rules! spawn_future_allow_threads {
	($fut:expr) => {
		$crate::ffi::python::tokio().spawn($crate::ffi::python::AllowThreads(Box::pin(
			async move {
				tracing::info!("running future from rust.");
				$fut.await
			},
		)))
	};
}

#[macro_export]
macro_rules! spawn_future {
	($fut:expr) => {
		$crate::ffi::python::tokio().spawn(async move { $fut.await })
	};
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

#[derive(Debug, Clone)]
struct LoggerProducer(watch::Sender<String>);

impl std::io::Write for LoggerProducer {
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		let _ = self.0.send(String::from_utf8_lossy(buf).to_string()); // ignore: logger disconnected or with full buffer
		Ok(buf.len())
	}

	fn flush(&mut self) -> std::io::Result<()> {
		Ok(())
	}
}

#[pyclass]
struct PyLogger(watch::Receiver<String>);

#[pymethods]
impl PyLogger {
	#[new]
	fn init_logger(debug: bool) -> PyResult<Self> {
		let (tx, mut rx) = watch::channel("logger initialised".to_string());
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

		match tracing_subscriber::fmt()
			.with_ansi(false)
			.event_format(format)
			.with_max_level(level)
			.with_writer(std::sync::Mutex::new(LoggerProducer(tx)))
			.try_init()
		{
			Ok(_) => Ok(PyLogger(rx)),
			Err(_) => Err(PySystemError::new_err("A logger already exists")),
		}
	}

	async fn listen(&mut self) -> Option<String> {
		if self.0.changed().await.is_ok() {
			return Some(self.0.borrow().clone());
		}

		None
	}
}

#[pyclass]
pub struct Promise(Option<tokio::task::JoinHandle<PyResult<PyObject>>>);

#[pymethods]
impl Promise {
	#[pyo3(name = "await")]
	fn a_wait(&mut self) -> PyResult<PyObject> {
		match self.0.take() {
			None => Err(PySystemError::new_err(
				"promise can't be awaited multiple times!",
			)),
			Some(x) => match tokio().block_on(x) {
				Err(e) => Err(PySystemError::new_err(format!(
					"error awaiting promise: {e}"
				))),
				Ok(res) => res,
			},
		}
	}
}

#[pymodule]
fn codemp(m: &Bound<'_, PyModule>) -> PyResult<()> {
	m.add_class::<PyLogger>()?;

	m.add_class::<TextChange>()?;
	m.add_class::<BufferController>()?;

	m.add_class::<Cursor>()?;
	m.add_class::<CursorController>()?;

	m.add_class::<Workspace>()?;
	m.add_class::<Client>()?;

	Ok(())
}
