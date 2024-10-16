use super::a_sync_allow_threads;
use super::Client;
use crate::api::User;
use crate::workspace::Workspace;
use pyo3::prelude::*;

#[pymethods]
impl Client {
	// #[new]
	// fn __new__(
	// 	host: String,
	// 	username: String,
	// 	password: String,
	// ) -> crate::errors::ConnectionResult<Self> {
	// 	super::tokio().block_on(Client::connect(host, username, password))
	// }

	#[pyo3(name = "attach_workspace")]
	fn pyattach_workspace(&self, py: Python<'_>, workspace: String) -> PyResult<super::Promise> {
		tracing::info!("attempting to join the workspace {}", workspace);
		let this = self.clone();
		a_sync_allow_threads!(py, this.attach_workspace(workspace).await)
		// let this = self.clone();
		// Ok(super::Promise(Some(tokio().spawn(async move {
		// 	Ok(this
		// 		.join_workspace(workspace)
		// 		.await
		// 		.map(|f| Python::with_gil(|py| f.into_py(py)))?)
		// }))))
	}

	#[pyo3(name = "create_workspace")]
	fn pycreate_workspace(&self, py: Python<'_>, workspace: String) -> PyResult<super::Promise> {
		tracing::info!("attempting to create workspace {}", workspace);
		let this = self.clone();
		a_sync_allow_threads!(py, this.create_workspace(workspace).await)
	}

	#[pyo3(name = "delete_workspace")]
	fn pydelete_workspace(&self, py: Python<'_>, workspace: String) -> PyResult<super::Promise> {
		tracing::info!("attempting to delete workspace {}", workspace);
		let this = self.clone();
		a_sync_allow_threads!(py, this.delete_workspace(workspace).await)
	}

	#[pyo3(name = "invite_to_workspace")]
	fn pyinvite_to_workspace(
		&self,
		py: Python<'_>,
		workspace: String,
		user: String,
	) -> PyResult<super::Promise> {
		tracing::info!("attempting to invite {user} to workspace {workspace}");
		let this = self.clone();
		a_sync_allow_threads!(py, this.invite_to_workspace(workspace, user).await)
	}

	#[pyo3(name = "fetch_owned_workspaces")]
	fn pyfetch_owned_workspaces(&self, py: Python<'_>) -> PyResult<super::Promise> {
		tracing::info!("attempting to fetch owned workspaces");
		let this = self.clone();
		a_sync_allow_threads!(py, this.fetch_owned_workspaces().await)
	}

	#[pyo3(name = "fetch_joined_workspaces")]
	fn pyfetch_joined_workspaces(&self, py: Python<'_>) -> PyResult<super::Promise> {
		tracing::info!("attempting to fetch joined workspaces");
		let this = self.clone();
		a_sync_allow_threads!(py, this.fetch_joined_workspaces().await)
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

	#[pyo3(name = "current_user")]
	fn pycurrent_user(&self) -> User {
		self.current_user().clone()
	}

	#[pyo3(name = "refresh")]
	fn pyrefresh(&self, py: Python<'_>) -> PyResult<super::Promise> {
		tracing::info!("attempting to refresh token");
		let this = self.clone();
		a_sync_allow_threads!(py, this.refresh().await)
	}
}
