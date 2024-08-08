use crate::api::Controller;
use crate::api::Cursor;
use crate::api::TextChange;
use crate::buffer::Controller as BufferController;
use crate::cursor::Controller as CursorController;
use pyo3::prelude::*;
use pyo3::types::PyType;

// use super::CodempController;

#[pymethods]
impl CursorController {
	#[pyo3(name = "send")]
	pub fn pysend(&self, path: String, start: (i32, i32), end: (i32, i32)) -> PyResult<()> {
		let pos = Cursor {
			start,
			end,
			buffer: path,
			user: None,
		};

		Ok(self.send(pos)?)
	}

	#[pyo3(name = "try_recv")]
	pub fn pytry_recv(&self, py: Python<'_>) -> PyResult<Option<Py<Cursor>>> {
		match self.try_recv()? {
			Some(cur_event) => Ok(Some(Py::new(py, cur_event)?)),
			None => Ok(None),
		}
	}

	#[pyo3(name = "recv")]
	pub fn pyrecv<'p>(&'p self, py: Python<'p>) -> PyResult<&'p PyAny> {
		let rc = self.clone();

		pyo3_asyncio::tokio::future_into_py(py, async move {
			let cur_event: Cursor = rc.recv().await?;
			Python::with_gil(|py| Py::new(py, cur_event))
		})
	}

	#[pyo3(name = "poll")]
	pub fn pypoll<'p>(&'p self, py: Python<'p>) -> PyResult<&'p PyAny> {
		let rc = self.clone();

		pyo3_asyncio::tokio::future_into_py(py, async move { Ok(rc.poll().await?) })
	}

	#[pyo3(name = "stop")]
	pub fn pystop(&self) -> bool {
		self.stop()
	}
}

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
	fn pyuser(&self) -> String {
		match self.user {
			Some(user) => user.to_string(),
			None => "".to_string(),
		}
	}
}

#[pymethods]
impl BufferController {
	#[pyo3(name = "content")]
	fn pycontent(&self) -> String {
		self.content().clone()
	}

	#[pyo3(name = "send")]
	fn pysend(&self, start: u32, end: u32, txt: String) -> PyResult<()> {
		let op = TextChange {
			start,
			end,
			content: txt,
		};
		Ok(self.send(op)?)
	}

	#[pyo3(name = "try_recv")]
	fn pytry_recv(&self, py: Python<'_>) -> PyResult<PyObject> {
		match self.try_recv()? {
			Some(txt_change) => {
				let evt = txt_change;
				Ok(evt.into_py(py))
			}
			None => Ok(py.None()),
		}
	}

	#[pyo3(name = "recv")]
	fn pyrecv<'p>(&'p self, py: Python<'p>) -> PyResult<&'p PyAny> {
		let rc = self.clone();

		pyo3_asyncio::tokio::future_into_py(py, async move {
			let txt_change: TextChange = rc.recv().await?;
			Python::with_gil(|py| Py::new(py, txt_change))
		})
	}

	#[pyo3(name = "poll")]
	fn pypoll<'p>(&'p self, py: Python<'p>) -> PyResult<&'p PyAny> {
		let rc = self.clone();

		pyo3_asyncio::tokio::future_into_py(py, async move { Ok(rc.poll().await?) })
	}
}

#[pymethods]
impl TextChange {
	#[pyo3(name = "is_deletion")]
	fn pyis_deletion(&self) -> bool {
		self.is_deletion()
	}

	#[pyo3(name = "is_addition")]
	fn pyis_addition(&self) -> bool {
		self.is_addition()
	}

	#[pyo3(name = "is_empty")]
	fn pyis_empty(&self) -> bool {
		self.is_empty()
	}

	#[pyo3(name = "apply")]
	fn pyapply(&self, txt: &str) -> String {
		self.apply(txt)
	}

	#[classmethod]
	#[pyo3(name = "from_diff")]
	fn pyfrom_diff(_cls: &PyType, before: &str, after: &str) -> TextChange {
		TextChange::from_diff(before, after)
	}

	#[classmethod]
	#[pyo3(name = "index_to_rowcol")]
	fn pyindex_to_rowcol(_cls: &PyType, txt: &str, index: usize) -> (i32, i32) {
		TextChange::index_to_rowcol(txt, index).into()
	}
}
