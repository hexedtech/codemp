use crate::api::Controller;
use crate::api::Cursor;
use crate::api::TextChange;
use crate::buffer::Controller as BufferController;
use crate::cursor::Controller as CursorController;
use pyo3::prelude::*;
use pyo3::types::PyFunction;

use super::Promise;
use crate::a_sync_allow_threads;

// need to do manually since Controller is a trait implementation
#[pymethods]
impl CursorController {
	#[pyo3(name = "send")]
	fn pysend(
		&self,
		py: Python,
		path: String,
		start: (i32, i32),
		end: (i32, i32),
	) -> PyResult<Promise> {
		let pos = Cursor {
			start,
			end,
			buffer: path,
			user: None,
		};
		let this = self.clone();
		a_sync_allow_threads!(py, this.send(pos).await)
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
	fn pycallback(&self, py: Python, cb: Py<PyFunction>) {
		py.allow_threads(move || {
			self.callback(move |ctl| {
				Python::with_gil(|py| {
					// TODO what to do with this error?
					let _ = cb.call1(py, (ctl,));
				})
			})
		})
	}

	#[pyo3(name = "clear_callback")]
	fn pyclear_callback(&self) {
		self.clear_callback();
	}

	#[pyo3(name = "stop")]
	fn pystop(&self) -> bool {
		self.stop()
	}
}

// need to do manually since Controller is a trait implementation
#[pymethods]
impl BufferController {
	#[pyo3(name = "name")]
	fn pyname(&self) -> String {
		self.name().to_string()
	}

	#[pyo3(name = "content")]
	fn pycontent(&self, py: Python) -> PyResult<Promise> {
		let this = self.clone();
		a_sync_allow_threads!(py, this.content().await)
	}

	#[pyo3(name = "send")]
	fn pysend(&self, py: Python, start: u32, end: u32, txt: String) -> PyResult<Promise> {
		let op = TextChange {
			start,
			end,
			content: txt,
			hash: None,
		};
		let this = self.clone();
		a_sync_allow_threads!(py, this.send(op).await)
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
	fn pycallback(&self, py: Python, cb: Py<PyFunction>) {
		py.allow_threads(move || {
			self.callback(move |ctl| {
				Python::with_gil(|py| {
					// TODO what to do with this error?
					let _ = cb.call1(py, (ctl,));
				})
			})
		})
	}

	#[pyo3(name = "clear_callback")]
	fn pyclear_callback(&self) {
		self.clear_callback();
	}

	#[pyo3(name = "stop")]
	fn pystop(&self) -> bool {
		self.stop()
	}
}

// We have to write this manually since
// cursor.user has type Option which cannot be translated
// automatically
#[pymethods]
impl Cursor {
	#[getter(start)]
	fn pystart(&self) -> (i32, i32) {
		self.start
	}

	#[getter(end)]
	fn pyend(&self) -> (i32, i32) {
		self.end
	}

	#[getter(buffer)]
	fn pybuffer(&self) -> String {
		self.buffer.clone()
	}

	#[getter(user)]
	fn pyuser(&self) -> Option<String> {
		self.user.map(|user| user.to_string())
	}
}
