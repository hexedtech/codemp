use std::{format, sync::Arc};
use tokio::sync::{mpsc, Mutex, RwLock};
use tracing;
use tracing_subscriber;

use crate::prelude::*;

use pyo3::{
	exceptions::{PyConnectionError, PyRuntimeError},
	prelude::*,
	types::{PyList, PyString, PyTuple, PyType},
};

// ERRORS And LOGGING ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
impl From<CodempError> for PyErr {
	fn from(value: CodempError) -> Self {
		match value {
			CodempError::Transport { status, message } => {
				PyConnectionError::new_err(format!("Transport error: ({}) {}", status, message))
			}
			CodempError::Channel { send } => {
				PyConnectionError::new_err(format!("Channel error (send:{})", send))
			}
			CodempError::InvalidState { msg } => {
				PyRuntimeError::new_err(format!("Invalid state: {}", msg))
			}
			CodempError::Deadlocked => PyRuntimeError::new_err(format!("Deadlock, retry.")),
		}
	}
}

#[derive(Debug, Clone)]
struct LoggerProducer(mpsc::Sender<String>);

impl std::io::Write for LoggerProducer {
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		// TODO this is a LOSSY logger!!
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
	fn message<'a>(&'a self, py: Python<'a>) -> PyResult<&'a PyAny> {
		let rc = self.0.clone();

		pyo3_asyncio::tokio::future_into_py(py, async move { Ok(rc.lock().await.recv().await) })
	}
}
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

// Workflow:
// We first spin up an empty handler, that can connect to a server, and create a client.
// We then use the client, to login into a workspace, and join it, obtaining a workspace object
// We will then use that workspace object to interact with the buffers in that workspace.
// In steps:
//  1. Get Object that can initiate a connection
//  2. Connect to a server
//  3. Login to a workspace
//  4. Join a workspace/get an already joined workspace
//  5. Create a new buffer/attach to an existing one

#[pyfunction]
fn codemp_init<'a>(py: Python<'a>) -> PyResult<Py<Client>> {
	Ok(Py::new(py, Client::default())?)
}

#[pyfunction]
fn init_logger(py: Python<'_>, debug: Option<bool>) -> PyResult<Py<PyLogger>> {
	let (tx, rx) = mpsc::channel(256);
	let level = if debug.unwrap_or(false) {
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

	let _ = tracing_subscriber::fmt()
		.with_ansi(false)
		.event_format(format)
		.with_max_level(level)
		.with_writer(std::sync::Mutex::new(LoggerProducer(tx)))
		.try_init();

	Ok(Py::new(py, PyLogger(Arc::new(Mutex::new(rx))))?)
}

#[pyclass]
struct Client(Arc<RwLock<Option<CodempClient>>>);

impl Default for Client {
	fn default() -> Self {
		Client(Arc::new(RwLock::new(None)))
	}
}

impl From<CodempClient> for Client {
	fn from(value: CodempClient) -> Self {
		Client(RwLock::new(Some(value)).into())
	}
}

#[pymethods]
impl Client {
	fn connect<'a>(&'a self, py: Python<'a>, dest: String) -> PyResult<&'a PyAny> {
		let cli = self.0.clone();

		pyo3_asyncio::tokio::future_into_py(py, async move {
			let client: CodempClient = CodempClient::new(dest.as_str()).await?;

			let _ = cli.write().await.insert(client);

			Ok(())
		})
	}

	fn login<'a>(
		&'a self,
		py: Python<'a>,
		user: String,
		password: String,
		workspace_id: Option<String>,
	) -> PyResult<&'a PyAny> {
		let rc = self.0.clone();
		pyo3_asyncio::tokio::future_into_py(py, async move {
			let cli = rc.read().await;
			if cli.is_none() {
				return Err(PyConnectionError::new_err("Connect to a server first."));
			};

			cli.as_ref()
				.unwrap()
				.login(user, password, workspace_id)
				.await?;

			Ok(())
		})
	}

	fn join_workspace<'a>(&'a self, py: Python<'a>, workspace: String) -> PyResult<&'a PyAny> {
		let rc = self.0.clone();

		pyo3_asyncio::tokio::future_into_py(py, async move {
			let mut cli = rc.write().await;
			if cli.is_none() {
				return Err(PyConnectionError::new_err("Connect to a server first."));
			};

			let workspace: CodempWorkspace = cli
				.as_mut()
				.unwrap()
				.join_workspace(workspace.as_str())
				.await?
				.into();

			Python::with_gil(|py| Ok(Py::new(py, workspace)?))
		})
	}

	// join a workspace
	fn get_workspace<'a>(&'a self, py: Python<'a>, id: String) -> PyResult<&'a PyAny> {
		let rc = self.0.clone();

		pyo3_asyncio::tokio::future_into_py(py, async move {
			let cli = rc.read().await;
			if cli.is_none() {
				return Err(PyConnectionError::new_err("Connect to a server first."));
			};

			let Some(ws) = cli.as_ref().unwrap().get_workspace(id.as_str()) else {
				return Ok(None);
			};

			Python::with_gil(|py| Ok(Some(Py::new(py, ws)?)))
		})
	}

	fn user_id<'a>(&'a self, py: Python<'a>) -> PyResult<&'a PyAny> {
		let rc = self.0.clone();
		pyo3_asyncio::tokio::future_into_py(py, async move {
			let cli = rc.read().await;
			if cli.is_none() {
				return Err(PyConnectionError::new_err("Connect to a server first."));
			};
			let id = cli.as_ref().unwrap().user_id().to_string();

			Python::with_gil(|py| {
				let id: Py<PyString> = PyString::new(py, id.as_str()).into_py(py);
				Ok(id)
			})
		})
	}
}

#[pymethods]
impl CodempWorkspace {
	// join a workspace
	#[pyo3(name = "create")]
	fn pycreate<'a>(&'a self, py: Python<'a>, path: String) -> PyResult<&'a PyAny> {
		let ws = self.clone();

		pyo3_asyncio::tokio::future_into_py(py, async move {
			ws.create(path.as_str()).await?;
			Ok(())
		})
	}
	#[pyo3(name = "attach")]
	fn pyattach<'a>(&'a self, py: Python<'a>, path: String) -> PyResult<&'a PyAny> {
		let ws = self.clone();

		pyo3_asyncio::tokio::future_into_py(py, async move {
			let buffctl: CodempBufferController = ws.attach(path.as_str()).await?.into();
			Python::with_gil(|py| Ok(Py::new(py, buffctl)?))
		})
	}

	#[pyo3(name = "fetch_buffers")]
	fn pyfetch_buffers<'a>(&'a self, py: Python<'a>) -> PyResult<&'a PyAny> {
		let ws = self.clone();

		pyo3_asyncio::tokio::future_into_py(py, async move {
			ws.fetch_buffers().await?;
			Ok(())
		})
	}

	#[pyo3(name = "fetch_users")]
	fn pyfetch_users<'a>(&'a self, py: Python<'a>) -> PyResult<&'a PyAny> {
		let ws = self.clone();

		pyo3_asyncio::tokio::future_into_py(py, async move {
			ws.fetch_users().await?;
			Ok(())
		})
	}

	#[pyo3(name = "list_buffer_users")]
	fn pylist_buffer_users<'a>(&'a self, py: Python<'a>, path: String) -> PyResult<&'a PyAny> {
		let ws = self.clone();

		pyo3_asyncio::tokio::future_into_py(py, async move {
			let usrlist: Vec<String> = ws
				.list_buffer_users(path.as_str())
				.await?
				.into_iter()
				.map(|e| e.id)
				.collect();

			Ok(usrlist)
		})
	}

	#[pyo3(name = "delete")]
	fn pydelete<'a>(&'a self, py: Python<'a>, path: String) -> PyResult<&'a PyAny> {
		let ws = self.clone();

		pyo3_asyncio::tokio::future_into_py(py, async move {
			ws.delete(path.as_str()).await?;
			Ok(())
		})
	}

	#[pyo3(name = "id")]
	fn pyid(&self, py: Python<'_>) -> Py<PyString> {
		PyString::new(py, self.id().as_str()).into()
	}

	#[pyo3(name = "cursor")]
	fn pycursor(&self, py: Python<'_>) -> PyResult<Py<CodempCursorController>> {
		Ok(Py::new(py, CodempCursorController::from(self.cursor()))?)
	}

	#[pyo3(name = "buffer_by_name")]
	fn pybuffer_by_name(
		&self,
		py: Python<'_>,
		path: String,
	) -> PyResult<Option<Py<CodempBufferController>>> {
		let Some(bufctl) = self.buffer_by_name(path.as_str()) else {
			return Ok(None);
		};

		Ok(Some(Py::new(py, CodempBufferController::from(bufctl))?))
	}

	#[pyo3(name = "filetree")]
	fn pyfiletree(&self, py: Python<'_>) -> Py<PyList> {
		PyList::new(py, self.filetree()).into_py(py)
	}
}

/* ########################################################################### */

#[pymethods]
impl CodempCursorController {
	#[pyo3(name = "send")]
	fn pysend<'a>(&'a self, path: String, start: (i32, i32), end: (i32, i32)) -> PyResult<()> {
		let pos = CodempCursor {
			start: start.into(),
			end: end.into(),
			buffer: path,
			user: None,
		};

		Ok(self.send(pos)?)
	}

	#[pyo3(name = "try_recv")]
	fn pytry_recv(&self, py: Python<'_>) -> PyResult<PyObject> {
		match self.try_recv()? {
			Some(cur_event) => {
				let evt = CodempCursor::from(cur_event);
				Ok(evt.into_py(py))
			}
			None => Ok(py.None()),
		}
	}

	#[pyo3(name = "recv")]
	fn pyrecv<'a>(&'a self, py: Python<'a>) -> PyResult<&'a PyAny> {
		let rc = self.clone();

		pyo3_asyncio::tokio::future_into_py(py, async move {
			let cur_event: CodempCursor = rc.recv().await?.into();
			Python::with_gil(|py| Ok(Py::new(py, cur_event)?))
		})
	}

	#[pyo3(name = "poll")]
	fn pypoll<'a>(&'a self, py: Python<'a>) -> PyResult<&'a PyAny> {
		let rc = self.clone();

		pyo3_asyncio::tokio::future_into_py(py, async move { Ok(rc.poll().await?) })
	}
}

#[pymethods]
impl CodempCursor {
	#[getter(start)]
	fn pystart(&self, py: Python<'_>) -> Py<PyTuple> {
		self.start.into_py(py)
	}

	#[getter(end)]
	fn pyend(&self, py: Python<'_>) -> Py<PyTuple> {
		self.end.into_py(py)
	}

	#[getter(buffer)]
	fn pybuffer(&self, py: Python<'_>) -> Py<PyString> {
		PyString::new(py, self.buffer.as_str()).into()
	}

	#[getter(user)]
	fn pyuser(&self, py: Python<'_>) -> Py<PyString> {
		match self.user {
			Some(user) => PyString::new(py, user.to_string().as_str()).into(),
			None => "".into_py(py),
		}
	}
}

#[pymethods]
impl CodempBufferController {
	#[pyo3(name = "content")]
	fn pycontent<'a>(&self, py: Python<'a>) -> &'a PyString {
		PyString::new(py, self.content().as_str())
	}

	#[pyo3(name = "send")]
	fn pysend(&self, start: usize, end: usize, txt: String) -> PyResult<()> {
		let op = CodempTextChange {
			span: start..end,
			content: txt.into(),
		};
		Ok(self.send(op)?)
	}

	#[pyo3(name = "try_recv")]
	fn pytry_recv(&self, py: Python<'_>) -> PyResult<PyObject> {
		match self.try_recv()? {
			Some(txt_change) => {
				let evt = CodempTextChange::from(txt_change);
				Ok(evt.into_py(py))
			}
			None => Ok(py.None()),
		}
	}

	#[pyo3(name = "recv")]
	fn pyrecv<'a>(&'a self, py: Python<'a>) -> PyResult<&'a PyAny> {
		let rc = self.clone();

		pyo3_asyncio::tokio::future_into_py(py, async move {
			let txt_change: CodempTextChange = rc.recv().await?.into();
			Python::with_gil(|py| Ok(Py::new(py, txt_change)?))
		})
	}

	#[pyo3(name = "poll")]
	fn pypoll<'a>(&'a self, py: Python<'a>) -> PyResult<&'a PyAny> {
		let rc = self.clone();

		pyo3_asyncio::tokio::future_into_py(py, async move { Ok(rc.poll().await?) })
	}
}

#[pymethods]
impl CodempTextChange {
	#[getter]
	#[pyo3(name = "start_incl")]
	fn pystart_incl(&self) -> PyResult<usize> {
		Ok(self.span.start)
	}

	#[getter]
	#[pyo3(name = "end_excl")]
	fn pyend_excl(&self) -> PyResult<usize> {
		Ok(self.span.end)
	}

	#[getter]
	#[pyo3(name = "content")]
	fn pycontent(&self) -> PyResult<String> {
		Ok(self.content.clone())
	}

	#[pyo3(name = "is_deletion")]
	fn pyis_deletion(&self) -> bool {
		self.is_deletion()
	}

	#[pyo3(name = "is_addition")]
	fn pyis_addition(&self) -> bool {
		self.is_addition()
	}

	#[pyo3(name = "is_empty")]
	fn pyis_empty(&self) -> bool {
		self.is_empty()
	}

	#[pyo3(name = "apply")]
	fn pyapply(&self, txt: &str) -> String {
		self.apply(txt)
	}

	#[classmethod]
	#[pyo3(name = "from_diff")]
	fn pyfrom_diff(_cls: &PyType, before: &str, after: &str) -> CodempTextChange {
		CodempTextChange::from_diff(before, after)
	}

	#[classmethod]
	#[pyo3(name = "index_to_rowcol")]
	fn pyindex_to_rowcol(_cls: &PyType, txt: &str, index: usize) -> (i32, i32) {
		CodempTextChange::index_to_rowcol(txt, index).into()
	}
}

/* ------ Python module --------*/
#[pymodule]
fn codemp(_py: Python, m: &PyModule) -> PyResult<()> {
	m.add_function(wrap_pyfunction!(codemp_init, m)?)?;
	m.add_function(wrap_pyfunction!(init_logger, m)?)?;
	m.add_class::<Client>()?;
	m.add_class::<PyLogger>()?;
	m.add_class::<CodempWorkspace>()?;
	m.add_class::<CodempCursorController>()?;
	m.add_class::<CodempBufferController>()?;

	m.add_class::<CodempCursor>()?;
	m.add_class::<CodempTextChange>()?;

	Ok(())
}
