use crate::api::controller::{AsyncReceiver, AsyncSender};
use crate::api::TextChange;
use crate::api::{Cursor, Selection};
use crate::buffer::Controller as BufferController;
use crate::cursor::Controller as CursorController;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use super::a_sync_allow_threads;
use super::Promise;

// need to do manually since Controller is a trait implementation
#[pymethods]
impl CursorController {
	#[pyo3(name = "send")]
	fn pysend(&self, _py: Python, pos: Selection) -> PyResult<()> {
		self.send(pos)?;
		Ok(())
	}

	#[pyo3(name = "try_recv")]
	fn pytry_recv(&self, py: Python) -> PyResult<Promise> {
		let this = self.clone();
		a_sync_allow_threads!(py, this.try_recv().await)
	}

	#[pyo3(name = "recv")]
	fn pyrecv(&self, py: Python) -> PyResult<Promise> {
		let this = self.clone();
		a_sync_allow_threads!(py, this.recv().await)
	}

	#[pyo3(name = "poll")]
	fn pypoll(&self, py: Python) -> PyResult<Promise> {
		let this = self.clone();
		a_sync_allow_threads!(py, this.poll().await)
	}

	#[pyo3(name = "callback")]
	fn pycallback(&self, py: Python, cb: PyObject) -> PyResult<()> {
		if !cb.bind_borrowed(py).is_callable() {
			return Err(PyValueError::new_err("The object passed must be callable."));
		}

		self.callback(move |ctl| {
			Python::with_gil(|py| {
				// TODO what to do with this error?
				let _ = cb.call1(py, (ctl,));
			})
		});
		Ok(())
	}

	#[pyo3(name = "clear_callback")]
	fn pyclear_callback(&self) {
		self.clear_callback();
	}
}

// need to do manually since Controller is a trait implementation
#[pymethods]
impl BufferController {
	#[pyo3(name = "path")]
	fn pypath(&self) -> String {
		self.path().to_string()
	}

	#[pyo3(name = "content")]
	fn pycontent(&self, py: Python) -> PyResult<Promise> {
		let this = self.clone();
		a_sync_allow_threads!(py, this.content().await)
	}

	#[pyo3(name = "ack")]
	fn pyack(&self, py: Python, v: Vec<i64>) -> () {
		self.ack(v)
	}

	#[pyo3(name = "send")]
	fn pysend(&self, _py: Python, op: TextChange) -> PyResult<()> {
		let this = self.clone();
		this.send(op)?;
		Ok(())
	}

	#[pyo3(name = "try_recv")]
	fn pytry_recv(&self, py: Python) -> PyResult<Promise> {
		let this = self.clone();
		a_sync_allow_threads!(py, this.try_recv().await)
	}

	#[pyo3(name = "recv")]
	fn pyrecv(&self, py: Python) -> PyResult<Promise> {
		let this = self.clone();
		a_sync_allow_threads!(py, this.recv().await)
	}

	#[pyo3(name = "poll")]
	fn pypoll(&self, py: Python) -> PyResult<Promise> {
		let this = self.clone();
		a_sync_allow_threads!(py, this.poll().await)
	}

	#[pyo3(name = "callback")]
	fn pycallback(&self, py: Python, cb: PyObject) -> PyResult<()> {
		if !cb.bind_borrowed(py).is_callable() {
			return Err(PyValueError::new_err("The object passed must be callable."));
		}

		self.callback(move |ctl| {
			Python::with_gil(|py| {
				// TODO what to do with this error?
				let _ = cb.call1(py, (ctl,));
			})
		});
		Ok(())
	}

	#[pyo3(name = "clear_callback")]
	fn pyclear_callback(&self) {
		self.clear_callback();
	}
}

// We have to write this manually since
// cursor.user has type Option which cannot be translated
// automatically
#[pymethods]
impl Cursor {
	#[getter(start)]
	fn pystart(&self) -> (i32, i32) {
		(self.sel.start_row, self.sel.start_col)
	}

	#[getter(end)]
	fn pyend(&self) -> (i32, i32) {
		(self.sel.end_row, self.sel.end_col)
	}

	#[getter(buffer)]
	fn pybuffer(&self) -> String {
		self.sel.buffer.clone()
	}

	#[getter(user)]
	fn pyuser(&self) -> Option<String> {
		Some(self.user.clone())
	}
}
