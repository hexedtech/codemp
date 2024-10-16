pub mod client;
pub mod controllers;
pub mod workspace;

use crate::{
	api::{BufferUpdate, Config, Cursor, Selection, TextChange, User},
	buffer::Controller as BufferController,
	cursor::Controller as CursorController,
	Client, Workspace,
};

use pyo3::{
	exceptions::{PyConnectionError, PyRuntimeError, PySystemError},
	prelude::*,
	types::PyDict,
};

use std::sync::OnceLock;
use tokio::sync::{mpsc, oneshot};

// global reference to a current_thread tokio runtime
pub fn tokio() -> &'static tokio::runtime::Runtime {
	static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
	RT.get_or_init(|| {
		tokio::runtime::Builder::new_current_thread()
			.enable_all()
			.build()
			.unwrap()
	})
}

// #[pyfunction]
// fn register_event_loop(event_loop: PyObject) {
// 	static EVENT_LOOP: OnceLock<PyObject> = OnceLock::new();
// 	EVENT_LOOP.
// }

// #[pyfunction]
// fn setup_async(
// 	event_loop: PyObject,
// 	call_soon_thread_safe: PyObject, // asyncio.EventLoop.call_soon_threadsafe
// 	call_coroutine_thread_safe: PyObject, // asyncio.call_coroutine_threadsafe
// 	create_future: PyObject,         // asyncio.EventLoop.create_future
// ) {
// 	let _ = EVENT_LOOP.get_or_init(|| event_loop);
// 	let _ = CALL_SOON.get_or_init(|| call_soon_thread_safe);
// 	let _ = CREATE_TASK.get_or_init(|| call_coroutine_thread_safe);
// 	let _ = CREATE_FUTURE.get_or_init(|| create_future);
// }

#[pyclass]
pub struct Promise(Option<tokio::task::JoinHandle<PyResult<PyObject>>>);

#[pymethods]
impl Promise {
	// Can't use this in callbacks since tokio will complain about running
	// a runtime inside another runtime.
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

macro_rules! a_sync {
	($x:expr) => {{
		Ok($crate::ffi::python::Promise(Some(
			$crate::ffi::python::tokio()
				.spawn(async move { Ok($x.map(|f| Python::with_gil(|py| f.into_py(py)))?) }),
		)))
	}};
}
pub(crate) use a_sync;

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
pub(crate) use a_sync_allow_threads;

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

#[pymethods]
impl User {
	#[getter]
	fn get_id(&self) -> pyo3::PyResult<String> {
		Ok(self.id.to_string())
	}

	#[setter]
	fn set_id(&mut self, value: String) -> pyo3::PyResult<()> {
		self.id = value
			.parse()
			.map_err(|x: <uuid::Uuid as std::str::FromStr>::Err| {
				pyo3::exceptions::PyRuntimeError::new_err(x.to_string())
			})?;
		Ok(())
	}

	#[getter]
	fn get_name(&self) -> pyo3::PyResult<String> {
		Ok(self.name.clone())
	}

	#[setter]
	fn set_name(&mut self, value: String) -> pyo3::PyResult<()> {
		self.name = value;
		Ok(())
	}

	fn __str__(&self) -> String {
		format!("{self:?}")
	}
}

#[pymethods]
impl Config {
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

#[pymethods]
impl Cursor {
	fn __str__(&self) -> String {
		format!("{self:?}")
	}
}

#[pymethods]
impl Selection {
	#[new]
	#[pyo3(signature = (**kwds))]
	pub fn py_new(kwds: Option<&Bound<'_, PyDict>>) -> PyResult<Self> {
		if let Some(kwds) = kwds {
			let start_row = if let Some(e) = kwds.get_item("start_row")? {
				e.extract()?
			} else {
				0
			};

			let start_col = if let Some(e) = kwds.get_item("start_col")? {
				e.extract()?
			} else {
				0
			};

			let end_row = if let Some(e) = kwds.get_item("end_row")? {
				e.extract()?
			} else {
				0
			};

			let end_col = if let Some(e) = kwds.get_item("end_col")? {
				e.extract()?
			} else {
				0
			};

			let buffer = if let Some(e) = kwds.get_item("buffer")? {
				e.extract()?
			} else {
				String::default()
			};

			Ok(Self {
				start_row,
				start_col,
				end_row,
				end_col,
				buffer,
			})
		} else {
			Ok(Self::default())
		}
	}

	fn __str__(&self) -> String {
		format!("{self:?}")
	}
}

#[pymethods]
impl BufferUpdate {
	fn __str__(&self) -> String {
		format!("{self:?}")
	}
}

#[pymethods]
impl TextChange {
	#[new]
	#[pyo3(signature = (**kwds))]
	pub fn py_new(kwds: Option<&Bound<'_, PyDict>>) -> PyResult<Self> {
		if let Some(kwds) = kwds {
			let start_idx = if let Some(e) = kwds.get_item("start")? {
				e.extract()?
			} else {
				0
			};

			let end_idx = if let Some(e) = kwds.get_item("end")? {
				e.extract()?
			} else {
				0
			};

			let content = if let Some(e) = kwds.get_item("content")? {
				e.extract()?
			} else {
				String::default()
			};

			Ok(Self {
				start_idx,
				end_idx,
				content,
			})
		} else {
			Ok(Self::default())
		}
	}

	fn __str__(&self) -> String {
		format!("{self:?}")
	}
}

#[pyfunction]
fn connect(py: Python, config: Py<Config>) -> PyResult<Promise> {
	let conf: Config = config.extract(py)?;
	a_sync!(Client::connect(conf).await)
}

#[pyfunction]
fn set_logger(py: Python, logging_cb: PyObject, debug: bool) -> bool {
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
		.with_thread_names(false)
		.with_file(false)
		.with_line_number(false)
		.with_source_location(false)
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
				let _ = Python::with_gil(|py| logging_cb.call1(py, (msg,)));
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

#[pymodule]
fn codemp(m: &Bound<'_, PyModule>) -> PyResult<()> {
	m.add_function(wrap_pyfunction!(version, m)?)?;
	m.add_function(wrap_pyfunction!(init, m)?)?;
	m.add_function(wrap_pyfunction!(connect, m)?)?;
	m.add_function(wrap_pyfunction!(set_logger, m)?)?;
	m.add_class::<Driver>()?;

	m.add_class::<BufferUpdate>()?;
	m.add_class::<TextChange>()?;
	m.add_class::<BufferController>()?;

	m.add_class::<Cursor>()?;
	m.add_class::<Selection>()?;
	m.add_class::<CursorController>()?;

	m.add_class::<User>()?;

	m.add_class::<Workspace>()?;
	m.add_class::<Client>()?;
	m.add_class::<Config>()?;

	Ok(())
}
