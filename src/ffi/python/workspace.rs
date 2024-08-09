use crate::buffer::Controller as BufferController;
use crate::cursor::Controller as CursorController;
use crate::workspace::Workspace;
use pyo3::prelude::*;
use pyo3::types::PyString;

#[pymethods]
impl Workspace {
	// join a workspace
	#[pyo3(name = "create")]
	fn pycreate<'p>(&'p self, py: Python<'p>, path: String) -> PyResult<&'p PyAny> {
		let ws = self.clone();

		pyo3_asyncio::tokio::future_into_py(py, async move {
			ws.create(path.as_str()).await?;
			Ok(())
		})
	}
	#[pyo3(name = "attach")]
	fn pyattach<'p>(&'p self, py: Python<'p>, path: String) -> PyResult<&PyAny> {
		let ws = self.clone();

		pyo3_asyncio::tokio::future_into_py(py, async move {
			let buffctl: BufferController = ws.attach(path.as_str()).await?;
			Python::with_gil(|py| Py::new(py, buffctl))
		})
	}

	#[pyo3(name = "detach")]
	fn pydetach(&self, path: String) -> bool {
		match self.detach(path.as_str()) {
			crate::workspace::worker::DetachResult::NotAttached => false,
			crate::workspace::worker::DetachResult::Detaching => true,
			crate::workspace::worker::DetachResult::AlreadyDetached => true,
		}
	}

	#[pyo3(name = "fetch_buffers")]
	fn pyfetch_buffers<'p>(&'p self, py: Python<'p>) -> PyResult<&PyAny> {
		let ws = self.clone();

		pyo3_asyncio::tokio::future_into_py(py, async move {
			ws.fetch_buffers().await?;
			Ok(())
		})
	}

	#[pyo3(name = "fetch_users")]
	fn pyfetch_users<'p>(&'p self, py: Python<'p>) -> PyResult<&PyAny> {
		let ws = self.clone();

		pyo3_asyncio::tokio::future_into_py(py, async move {
			ws.fetch_users().await?;
			Ok(())
		})
	}

	#[pyo3(name = "list_buffer_users")]
	fn pylist_buffer_users<'p>(&'p self, py: Python<'p>, path: String) -> PyResult<&PyAny> {
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
	fn pydelete<'p>(&'p self, py: Python<'p>, path: String) -> PyResult<&PyAny> {
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
	fn pycursor(&self, py: Python<'_>) -> PyResult<Py<CursorController>> {
		Py::new(py, self.cursor())
	}

	#[pyo3(name = "buffer_by_name")]
	fn pybuffer_by_name(
		&self,
		py: Python<'_>,
		path: String,
	) -> PyResult<Option<Py<BufferController>>> {
		let Some(bufctl) = self.buffer_by_name(path.as_str()) else {
			return Ok(None);
		};

		Ok(Some(Py::new(py, bufctl)?))
	}

	#[pyo3(name = "buffer_list")]
	fn pybuffer_list(&self) -> Vec<String> {
		self.buffer_list()
	}

	#[pyo3(name = "filetree")]
	fn pyfiletree(&self) -> Vec<String> {
		self.filetree()
	}
}
