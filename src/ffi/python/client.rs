use super::a_sync_allow_threads;
use super::Client;
use crate::workspace::Workspace;
use pyo3::prelude::*;

#[pymethods]
impl Client {
	#[new]
	fn __new__(
		host: String,
		username: String,
		password: String,
	) -> crate::errors::ConnectionResult<Self> {
		super::tokio().block_on(Client::connect(host, username, password))
	}

	// #[pyo3(name = "join_workspace")]
	// async fn pyjoin_workspace(&self, workspace: String) -> JoinHandle<crate::Result<Workspace>> {
	// 	tracing::info!("attempting to join the workspace {}", workspace);

	// 	let this = self.clone();
	// 	async {
	// 		tokio()
	// 			.spawn(async move { this.join_workspace(workspace).await })
	// 			.await
	// 	}
	// }

	#[pyo3(name = "join_workspace")]
	fn pyjoin_workspace(&self, py: Python<'_>, workspace: String) -> PyResult<super::Promise> {
		tracing::info!("attempting to join the workspace {}", workspace);
		let this = self.clone();
		a_sync_allow_threads!(py, this.join_workspace(workspace).await)
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

	#[pyo3(name = "list_workspaces")]
	fn pylist_workspaces(
		&self,
		py: Python<'_>,
		owned: bool,
		invited: bool,
	) -> PyResult<super::Promise> {
		tracing::info!("attempting to list workspaces");
		let this = self.clone();
		a_sync_allow_threads!(py, this.list_workspaces(owned, invited).await)
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
		self.user().id.to_string()
	}

	#[pyo3(name = "user_name")]
	fn pyuser_name(&self) -> String {
		self.user().name.clone()
	}

	#[pyo3(name = "refresh")]
	fn pyrefresh(&self, py: Python<'_>) -> PyResult<super::Promise> {
		tracing::info!("attempting to refresh token");
		let this = self.clone();
		a_sync_allow_threads!(py, this.refresh().await)
	}
}
