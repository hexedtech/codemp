use crate::workspace::Workspace;
use crate::Client;
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyList, PyString};

// #[pyfunction]
// pub fn codemp_init<'a>(py: Python<'a>) -> PyResult<Py<Client>> {
// 	Ok(Py::new(py, Client::default())?)
// }

#[pymethods]
impl Client {
	#[new]
	fn pyconnect(host: String, username: String, password: String) -> PyResult<Self> {
		let cli =
			pyo3_asyncio::tokio::get_runtime().block_on(Client::new(host, username, password));
		Ok(cli?)
	}

	#[pyo3(name = "join_workspace")]
	fn pyjoin_workspace<'a>(&'a self, py: Python<'a>, workspace: String) -> PyResult<&PyAny> {
		let rc = self.clone();

		pyo3_asyncio::tokio::future_into_py(py, async move {
			let workspace: Workspace = rc.join_workspace(workspace.as_str()).await?;
			Python::with_gil(|py| Py::new(py, workspace))
		})
	}

	#[pyo3(name = "leave_workspace")]
	fn pyleave_workspace<'p>(&'p self, py: Python<'p>, id: String) -> &PyBool {
		PyBool::new(py, self.leave_workspace(id.as_str()))
	}

	// join a workspace
	#[pyo3(name = "get_workspace")]
	fn pyget_workspace(&self, py: Python<'_>, id: String) -> PyResult<Option<Py<Workspace>>> {
		match self.get_workspace(id.as_str()) {
			Some(ws) => Ok(Some(Py::new(py, ws)?)),
			None => Ok(None),
		}
	}

	#[pyo3(name = "active_workspaces")]
	fn pyactive_workspaces<'p>(&'p self, py: Python<'p>) -> &PyList {
		PyList::new(py, self.active_workspaces())
	}

	#[pyo3(name = "user_id")]
	fn pyuser_id<'p>(&'p self, py: Python<'p>) -> &PyString {
		PyString::new(py, self.user_id().to_string().as_str())
	}
}
