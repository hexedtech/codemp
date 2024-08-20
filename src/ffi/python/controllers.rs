use crate::api::Controller;
use crate::api::Cursor;
use crate::api::TextChange;
use crate::buffer::Controller as BufferController;
use crate::cursor::Controller as CursorController;
use pyo3::prelude::*;

use crate::spawn_future;

// need to do manually since Controller is a trait implementation
#[pymethods]
impl CursorController {
	#[pyo3(name = "send")]
	async fn pysend(&self, path: String, start: (i32, i32), end: (i32, i32)) -> crate::Result<()> {
		let pos = Cursor {
			start,
			end,
			buffer: path,
			user: None,
		};
		let rc = self.clone();
		spawn_future!(rc.send(pos)).await.unwrap()
	}

	#[pyo3(name = "try_recv")]
	async fn pytry_recv(&self) -> crate::Result<Option<Cursor>> {
		let rc = self.clone();
		spawn_future!(rc.try_recv()).await.unwrap()
	}

	#[pyo3(name = "recv")]
	async fn pyrecv(&self) -> crate::Result<Cursor> {
		let rc = self.clone();
		spawn_future!(rc.recv()).await.unwrap()
	}

	#[pyo3(name = "poll")]
	async fn pypoll(&self) -> crate::Result<()> {
		let rc = self.clone();
		spawn_future!(rc.poll()).await.unwrap()
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
	async fn pycontent(&self) -> crate::Result<String> {
		let rc = self.clone();
		spawn_future!(rc.content()).await.unwrap()
	}

	#[pyo3(name = "send")]
	async fn pysend(&self, start: u32, end: u32, txt: String) -> crate::Result<()> {
		let op = TextChange {
			start,
			end,
			content: txt,
			hash: None,
		};
		let rc = self.clone();
		spawn_future!(rc.send(op)).await.unwrap()
	}

	#[pyo3(name = "try_recv")]
	async fn pytry_recv(&self) -> crate::Result<Option<TextChange>> {
		let rc = self.clone();
		spawn_future!(rc.try_recv()).await.unwrap()
	}

	#[pyo3(name = "recv")]
	async fn pyrecv(&self) -> crate::Result<TextChange> {
		let rc = self.clone();
		spawn_future!(rc.recv()).await.unwrap()
	}

	#[pyo3(name = "poll")]
	async fn pypoll(&self) -> crate::Result<()> {
		let rc = self.clone();
		spawn_future!(rc.poll()).await.unwrap()
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
