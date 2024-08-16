use crate::workspace::Workspace;
use crate::Client;
use pyo3::prelude::*;

// #[pyfunction]
// pub fn codemp_init<'a>(py: Python<'a>) -> PyResult<Py<Client>> {
// 	Ok(Py::new(py, Client::default())?)
// }

#[pymethods]
impl Client {
	#[new]
	async fn pyconnect(host: String, username: String, password: String) -> PyResult<Self> {
		Ok(Client::new(host, username, password));
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
	fn pyleave_workspace(&self, id: String) -> bool {
		self.leave_workspace(id.as_str())
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
	fn pyactive_workspaces(&self) -> Vec<String> {
		self.active_workspaces()
	}

	#[pyo3(name = "user_id")]
	fn pyuser_id(&self) -> String {
		self.user_id().to_string()
	}
}
