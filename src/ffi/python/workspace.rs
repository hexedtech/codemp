use crate::buffer::Controller as BufferController;
use crate::cursor::Controller as CursorController;
use crate::workspace::Workspace;
use pyo3::prelude::*;

use super::Promise;
use crate::a_sync;
// use super::Promise;

#[pymethods]
impl Workspace {
	// join a workspace
	#[pyo3(name = "create")]
	fn pycreate(&self, path: String) -> PyResult<Promise> {
		let this = self.clone();
		a_sync!(this.create(path.as_str()).await)
	}

	#[pyo3(name = "attach")]
	fn pyattach(&self, path: String) -> PyResult<Promise> {
		let this = self.clone();
		a_sync!(this.attach(path.as_str()).await)
	}

	#[pyo3(name = "detach")]
	fn pydetach(&self, path: String) -> bool {
		match self.detach(path.as_str()) {
			crate::workspace::worker::DetachResult::NotAttached => false,
			crate::workspace::worker::DetachResult::Detaching => true,
			crate::workspace::worker::DetachResult::AlreadyDetached => true,
		}
	}

	#[pyo3(name = "event")]
	fn pyevent(&self) -> PyResult<Promise> {
		let this = self.clone();
		a_sync!(this.event().await)
	}

	#[pyo3(name = "fetch_buffers")]
	fn pyfetch_buffers(&self) -> PyResult<Promise> {
		let this = self.clone();
		a_sync!(this.fetch_buffers().await)
	}

	#[pyo3(name = "fetch_users")]
	fn pyfetch_users(&self) -> PyResult<Promise> {
		let this = self.clone();
		a_sync!(this.fetch_users().await)
	}

	#[pyo3(name = "list_buffer_users")]
	fn pylist_buffer_users(&self, path: String) -> PyResult<Promise> {
		// crate::Result<Vec<crate::api::User>> {
		let this = self.clone();
		a_sync!(this.list_buffer_users(path.as_str()).await)
	}

	#[pyo3(name = "delete")]
	fn pydelete(&self, path: String) -> PyResult<Promise> {
		let this = self.clone();
		a_sync!(this.delete(path.as_str()).await)
	}

	#[pyo3(name = "id")]
	fn pyid(&self) -> String {
		self.id()
	}

	#[pyo3(name = "cursor")]
	fn pycursor(&self) -> CursorController {
		self.cursor()
	}

	#[pyo3(name = "buffer_by_name")]
	fn pybuffer_by_name(&self, path: String) -> Option<BufferController> {
		self.buffer_by_name(path.as_str())
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
