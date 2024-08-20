use crate::workspace::Workspace;
use crate::Client;
use pyo3::prelude::*;

use super::tokio;

#[pymethods]
impl Client {
	#[new]
	fn __new__(host: String, username: String, password: String) -> crate::Result<Self> {
		tokio().block_on(Client::new(host, username, password))
	}

	#[pyo3(name = "join_workspace")]
	fn pyjoin_workspace(&self, workspace: String) -> PyResult<super::RustPromise> {
		tracing::info!("attempting to join the workspace {workspace}");

		// crate::a_sync! { self => self.join_workspace(workspace).await }
		let rc = self.clone();
		Ok(super::RustPromise(Some(tokio().spawn(async move {
			Ok(rc
				.join_workspace(workspace)
				.await
				.map(|f| Python::with_gil(|py| f.into_py(py)))?)
		}))))
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
