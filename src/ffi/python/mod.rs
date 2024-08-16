pub mod client;
pub mod controllers;
pub mod workspace;

use std::sync::Arc;

use crate::{
	api::{Cursor, TextChange},
	buffer::Controller as BufferController,
	cursor::Controller as CursorController,
	Client, Workspace,
};
use pyo3::exceptions::{PyConnectionError, PyRuntimeError, PySystemError};
use pyo3::prelude::*;
use tokio::sync::{mpsc, Mutex};

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

#[derive(Debug, Clone)]
struct LoggerProducer(mpsc::Sender<String>);

impl std::io::Write for LoggerProducer {
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		let _ = self.0.try_send(String::from_utf8_lossy(buf).to_string()); // ignore: logger disconnected or with full buffer
		Ok(buf.len())
	}

	fn flush(&mut self) -> std::io::Result<()> {
		Ok(())
	}
}

#[pyclass]
struct PyLogger(Arc<Mutex<mpsc::Receiver<String>>>);

#[pymethods]
impl PyLogger {
	#[new]
	fn init_logger(debug: bool) -> PyResult<Self> {
		let (tx, rx) = mpsc::channel(256);
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
			Ok(_) => Ok(PyLogger(Arc::new(Mutex::new(rx)))),
			Err(_) => Err(PySystemError::new_err("A logger already exists")),
		}
	}

	async fn listen(&self) -> Option<String> {
		self.0.lock().await.recv().await
	}
}

#[pymodule]
fn codemp(_py: Python, m: &PyModule) -> PyResult<()> {
	m.add_class::<PyLogger>()?;

	m.add_class::<TextChange>()?;
	m.add_class::<BufferController>()?;

	m.add_class::<Cursor>()?;
	m.add_class::<CursorController>()?;

	m.add_class::<Workspace>()?;
	m.add_class::<Client>()?;

	Ok(())
}
