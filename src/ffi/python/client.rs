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

	async fn dioboia(&self) {
		tokio().spawn(async { tracing::info!("dioboia? si dioboia!") });
	}

	#[pyo3(name = "join_workspace")]
	async fn pyjoin_workspace(&self, workspace: String) -> crate::Result<Workspace> {
		// self.join_workspace(workspace).await
		let rc = self.clone();
		crate::spawn_future!(rc.join_workspace(workspace))
			.await
			.unwrap()
		// This expands to if spawn_future_allow_threads! is used
		// tokio()
		// 	.spawn(super::AllowThreads(Box::pin(async move {
		// 		rc.join_workspace(workspace).await
		// 	})))
		// or if only spawn_future!
		// tokio()
		// 	.spawn(async move { rc.join_workspace(workspace).await })
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
