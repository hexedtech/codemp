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
	fn pyconnect(host: String, username: String, password: String) -> crate::Result<Self> {
		super::tokio().block_on(async move { Client::new(host, username, password).await })
	}

	#[pyo3(name = "join_workspace")]
	async fn pyjoin_workspace(&self, workspace: String) -> crate::Result<Workspace> {
		self.join_workspace(workspace).await
	}

	#[pyo3(name = "leave_workspace")]
	fn pyleave_workspace(&self, id: String) -> bool {
		self.leave_workspace(id.as_str())
	}

	// join a workspace
	#[pyo3(name = "get_workspace")]
	fn pyget_workspace(&self, id: String) -> Option<Workspace> {
		self.get_workspace(id.as_str())
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
