// We no can simply define decorate the pub impl block of workspace with
// #[pymethods], which means that this module is not really necessary anymore.
// In any case we will leave it here for the time being in case we need to come back to
// the manual version.

// use crate::buffer::Controller as BufferController;
// use crate::cursor::Controller as CursorController;
// use crate::workspace::Workspace;
// use pyo3::prelude::*;

// #[pymethods]
// impl Workspace {
// 	// join a workspace
// 	#[pyo3(name = "create")]
// 	async fn pycreate(&self, path: String) -> crate::Result<()> {
// 		self.create(path.as_str()).await
// 	}

// 	#[pyo3(name = "attach")]
// 	async fn pyattach(&self, path: String) -> crate::Result<BufferController> {
// 		Ok(self.attach(path.as_str()).await?)
// 	}

// 	#[pyo3(name = "detach")]
// 	fn pydetach(&self, path: String) -> bool {
// 		match self.detach(path.as_str()) {
// 			crate::workspace::worker::DetachResult::NotAttached => false,
// 			crate::workspace::worker::DetachResult::Detaching => true,
// 			crate::workspace::worker::DetachResult::AlreadyDetached => true,
// 		}
// 	}

// 	#[pyo3(name = "event")]
// 	async fn pyevent(&self) -> crate::Result<crate::api::Event> {
// 		self.event().await
// 	}

// 	#[pyo3(name = "fetch_buffers")]
// 	async fn pyfetch_buffers(&self) -> crate::Result<()> {
// 		self.fetch_buffers().await
// 	}

// 	#[pyo3(name = "fetch_users")]
// 	async fn pyfetch_users(&self) -> crate::Result<()> {
// 		self.fetch_users().await
// 	}

// 	#[pyo3(name = "list_buffer_users")]
// 	async fn pylist_buffer_users(&self, path: String) -> crate::Result<Vec<crate::api::User>> {
// 		self.list_buffer_users(path.as_str()).await
// 	}

// 	#[pyo3(name = "delete")]
// 	async fn pydelete(&self, path: String) -> crate::Result<()> {
// 		self.delete(path.as_str()).await
// 	}

// 	#[pyo3(name = "id")]
// 	fn pyid(&self) -> String {
// 		self.id()
// 	}

// 	#[pyo3(name = "cursor")]
// 	fn pycursor(&self) -> CursorController {
// 		self.cursor()
// 	}

// 	#[pyo3(name = "buffer_by_name")]
// 	fn pybuffer_by_name(&self, path: String) -> Option<BufferController> {
// 		self.buffer_by_name(path.as_str())
// 	}

// 	#[pyo3(name = "buffer_list")]
// 	fn pybuffer_list(&self) -> Vec<String> {
// 		self.buffer_list()
// 	}

// 	#[pyo3(name = "filetree")]
// 	fn pyfiletree(&self) -> Vec<String> {
// 		self.filetree()
// 	}
// }
