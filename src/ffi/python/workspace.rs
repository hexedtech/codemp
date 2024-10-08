use crate::api::controller::AsyncReceiver;
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
	#[pyo3(name = "create")]
	fn pycreate(&self, py: Python, path: String) -> PyResult<Promise> {
		let this = self.clone();
		a_sync_allow_threads!(py, this.create(path.as_str()).await)
	}

	#[pyo3(name = "attach")]
	fn pyattach(&self, py: Python, path: String) -> PyResult<Promise> {
		let this = self.clone();
		a_sync_allow_threads!(py, this.attach(path.as_str()).await)
	}

	#[pyo3(name = "detach")]
	fn pydetach(&self, path: String) -> bool {
		self.detach(path.as_str())
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

	#[pyo3(name = "list_buffer_users")]
	fn pylist_buffer_users(&self, py: Python, path: String) -> PyResult<Promise> {
		// crate::Result<Vec<crate::api::User>>
		let this = self.clone();
		a_sync_allow_threads!(py, this.list_buffer_users(path.as_str()).await)
	}

	#[pyo3(name = "delete")]
	fn pydelete(&self, py: Python, path: String) -> PyResult<Promise> {
		let this = self.clone();
		a_sync_allow_threads!(py, this.delete(path.as_str()).await)
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
	#[pyo3(signature = (filter=None, strict=false))]
	fn pyfiletree(&self, filter: Option<&str>, strict: bool) -> Vec<String> {
		self.filetree(filter, strict)
	}

	#[pyo3(name = "user_list")]
	fn pyuser_list(&self) -> Vec<String> {
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
	fn pyclear_callbacl(&self, _py: Python) {
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
