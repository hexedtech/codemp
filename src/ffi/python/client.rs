use super::Client;
use super::a_sync_detach;
use crate::api::User;
use crate::workspace::Workspace;
use pyo3::prelude::*;
use uuid::Uuid;

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
	fn pyattach_workspace(&self, py: Python<'_>, workspace: Uuid) -> PyResult<super::Promise> {
		tracing::info!("attempting to join the workspace {}", workspace);
		let this = self.clone();
		a_sync_detach!(py, this.attach_workspace(workspace).await)
		// let this = self.clone();
		// Ok(super::Promise(Some(tokio().spawn(async move {
		// 	Ok(this
		// 		.join_workspace(workspace)
		// 		.await
		// 		.map(|f| Python::attach(|py| f.into_py(py)))?)
		// }))))
	}

	#[pyo3(name = "create_workspace")]
	fn pycreate_workspace(&self, py: Python<'_>, workspace: String) -> PyResult<super::Promise> {
		tracing::info!("creating workspace {}", workspace);
		let this = self.clone();
		a_sync_detach!(py, this.create_workspace(workspace).await)
	}

	#[pyo3(name = "delete_workspace")]
	fn pydelete_workspace(&self, py: Python<'_>, workspace: String) -> PyResult<super::Promise> {
		tracing::info!("deleting workspace {}", workspace);
		let this = self.clone();
		a_sync_detach!(py, this.delete_workspace(workspace).await)
	}

	#[pyo3(name = "invite_to_workspace")]
	fn pyinvite_to_workspace(
		&self,
		py: Python<'_>,
		workspace: String,
		user: String,
	) -> PyResult<super::Promise> {
		tracing::info!("inviting {user} to workspace {workspace}");
		let this = self.clone();
		a_sync_detach!(py, this.invite_to_workspace(workspace, user).await)
	}

	#[pyo3(name = "fetch_owned_workspaces")]
	fn pyfetch_owned_workspaces(&self, py: Python<'_>) -> PyResult<super::Promise> {
		tracing::info!("fetching owned workspaces");
		let this = self.clone();
		a_sync_detach!(py, this.fetch_owned_workspaces().await)
	}

	#[pyo3(name = "fetch_joined_workspaces")]
	fn pyfetch_joined_workspaces(&self, py: Python<'_>) -> PyResult<super::Promise> {
		tracing::info!("fetching joined workspaces");
		let this = self.clone();
		a_sync_detach!(py, this.fetch_joined_workspaces().await)
	}

	#[pyo3(name = "leave_workspace")]
	fn pyleave_workspace(&self, id: Uuid) -> bool {
		self.leave_workspace(id)
	}

	// join a workspace
	#[pyo3(name = "get_workspace")]
	fn pyget_workspace(&self, id: Uuid) -> Option<Workspace> {
		self.get_workspace(id)
	}

	#[pyo3(name = "active_workspaces")]
	fn pyactive_workspaces(&self) -> Vec<Uuid> {
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
		a_sync_detach!(py, this.refresh().await)
	}
}
