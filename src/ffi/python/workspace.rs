use crate::api::controller::AsyncReceiver;
use crate::api::User;
use crate::buffer::Controller as BufferController;
use crate::cursor::Controller as CursorController;
use crate::workspace::Workspace;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use super::a_sync_allow_threads;
use super::Promise;

#[pymethods]
impl Workspace {
	// join a workspace
	#[pyo3(name = "create_buffer")]
	fn pycreate_buffer(&self, py: Python, path: String) -> PyResult<Promise> {
		let this = self.clone();
		a_sync_allow_threads!(py, this.create_buffer(path.as_str()).await)
	}

	#[pyo3(name = "attach_buffer")]
	fn pyattach_buffer(&self, py: Python, path: String) -> PyResult<Promise> {
		let this = self.clone();
		a_sync_allow_threads!(py, this.attach_buffer(path.as_str()).await)
	}

	#[pyo3(name = "detach_buffer")]
	fn pydetach_buffer(&self, path: String) -> bool {
		self.detach_buffer(path.as_str())
	}

	#[pyo3(name = "fetch_buffers")]
	fn pyfetch_buffers(&self, py: Python) -> PyResult<Promise> {
		let this = self.clone();
		a_sync_allow_threads!(py, this.fetch_buffers().await)
	}

	#[pyo3(name = "fetch_users")]
	fn pyfetch_users(&self, py: Python) -> PyResult<Promise> {
		let this = self.clone();
		a_sync_allow_threads!(py, this.fetch_users().await)
	}

	#[pyo3(name = "fetch_buffer_users")]
	fn pyfetch_buffer_users(&self, py: Python, path: String) -> PyResult<Promise> {
		// crate::Result<Vec<crate::api::User>>
		let this = self.clone();
		a_sync_allow_threads!(py, this.fetch_buffer_users(path.as_str()).await)
	}

	#[pyo3(name = "delete_buffer")]
	fn pydelete_buffer(&self, py: Python, path: String) -> PyResult<Promise> {
		let this = self.clone();
		a_sync_allow_threads!(py, this.delete_buffer(path.as_str()).await)
	}

	#[pyo3(name = "id")]
	fn pyid(&self) -> String {
		self.id()
	}

	#[pyo3(name = "cursor")]
	fn pycursor(&self) -> CursorController {
		self.cursor()
	}

	#[pyo3(name = "get_buffer")]
	fn pyget_buffer(&self, path: String) -> Option<BufferController> {
		self.get_buffer(path.as_str())
	}

	#[pyo3(name = "active_buffers")]
	fn pyactive_buffers(&self) -> Vec<String> {
		self.active_buffers()
	}

	#[pyo3(name = "search_buffers")]
	#[pyo3(signature = (filter=None))]
	fn pysearch_buffers(&self, filter: Option<&str>) -> Vec<String> {
		self.search_buffers(filter)
	}

	#[pyo3(name = "user_list")]
	fn pyuser_list(&self) -> Vec<User> {
		self.user_list()
	}

	#[pyo3(name = "recv")]
	fn pyrecv(&self, py: Python) -> PyResult<Promise> {
		let this = self.clone();
		a_sync_allow_threads!(py, this.recv().await)
	}

	#[pyo3(name = "try_recv")]
	fn pytry_recv(&self, py: Python) -> PyResult<Promise> {
		let this = self.clone();
		a_sync_allow_threads!(py, this.try_recv().await)
	}

	#[pyo3(name = "poll")]
	fn pypoll(&self, py: Python) -> PyResult<Promise> {
		let this = self.clone();
		a_sync_allow_threads!(py, this.poll().await)
	}

	#[pyo3(name = "clear_callback")]
	fn pyclear_callback(&self, _py: Python) {
		self.clear_callback();
	}

	#[pyo3(name = "callback")]
	fn pycallback(&self, py: Python, cb: PyObject) -> PyResult<()> {
		if !cb.bind_borrowed(py).is_callable() {
			return Err(PyValueError::new_err("The object passed must be callable."));
		}

		self.callback(move |ws| {
			Python::with_gil(|py| {
				// TODO what to do with this error?
				let _ = cb.call1(py, (ws,));
			})
		});
		Ok(())
	}
}
