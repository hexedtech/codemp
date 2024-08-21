use crate::api::Controller;
use crate::api::Cursor;
use crate::api::TextChange;
use crate::buffer::Controller as BufferController;
use crate::cursor::Controller as CursorController;
use pyo3::prelude::*;

use super::Promise;
use crate::a_sync;

// need to do manually since Controller is a trait implementation
#[pymethods]
impl CursorController {
	#[pyo3(name = "send")]
	fn pysend(&self, path: String, start: (i32, i32), end: (i32, i32)) -> PyResult<Promise> {
		let pos = Cursor {
			start,
			end,
			buffer: path,
			user: None,
		};
		let this = self.clone();
		a_sync!(this.send(pos).await)
	}

	#[pyo3(name = "try_recv")]
	fn pytry_recv(&self) -> PyResult<Promise> {
		let this = self.clone();
		a_sync!(this.try_recv().await)
	}

	#[pyo3(name = "recv")]
	fn pyrecv(&self) -> crate::Result<Option<Cursor>> {
		Ok(super::tokio().block_on(self.try_recv())?)
		// let this = self.clone();
		// a_sync!(this.recv().await)
	}

	#[pyo3(name = "poll")]
	fn pypoll(&self) -> PyResult<Promise> {
		let this = self.clone();
		a_sync!(this.poll().await)
	}

	#[pyo3(name = "stop")]
	fn pystop(&self) -> bool {
		self.stop()
	}
}

// need to do manually since Controller is a trait implementation
#[pymethods]
impl BufferController {
	#[pyo3(name = "content")]
	async fn pycontent(&self) -> PyResult<Promise> {
		let this = self.clone();
		a_sync!(this.content().await)
	}

	#[pyo3(name = "send")]
	async fn pysend(&self, start: u32, end: u32, txt: String) -> PyResult<Promise> {
		let op = TextChange {
			start,
			end,
			content: txt,
			hash: None,
		};
		let this = self.clone();
		a_sync!(this.send(op).await)
	}

	#[pyo3(name = "try_recv")]
	fn pytry_recv(&self) -> crate::Result<Option<TextChange>> {
		Ok(super::tokio().block_on(self.try_recv())?)
		// let this = self.clone();
		// a_sync!(this.try_recv().await)
	}

	#[pyo3(name = "recv")]
	async fn pyrecv(&self) -> PyResult<Promise> {
		let this = self.clone();
		a_sync!(this.recv().await)
	}

	#[pyo3(name = "poll")]
	async fn pypoll(&self) -> PyResult<Promise> {
		let this = self.clone();
		a_sync!(this.poll().await)
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
