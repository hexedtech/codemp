//! ### Cursor
//! Represents the position of a remote user's cursor.

#[cfg(feature = "py")]
use pyo3::prelude::*;

/// An event that occurred about a user's cursor.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "js", napi_derive::napi(object))]
#[cfg_attr(feature = "py", pyclass(get_all, from_py_object))]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
// #[cfg_attr(feature = "py", pyo3(crate = "reexported::pyo3"))]
pub struct CursorEvent {
	/// User who sent the cursor.
	pub user: String,
	/// Cursor position data
	pub cursor: Cursor,
}

/// A cursor instantaneous state
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "js", napi_derive::napi(object))]
#[cfg_attr(feature = "py", pyclass(get_all, from_py_object))]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
// #[cfg_attr(feature = "py", pyo3(crate = "reexported::pyo3"))]
pub struct Cursor {
	/// Path of buffer this cursor is on
	pub buffer: String,
	/// The updated cursor selection.
	pub sel: Vec<Selection>,
}

/// A cursor selection span.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "js", napi_derive::napi(object))]
#[cfg_attr(feature = "py", pyclass(get_all, from_py_object))]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
// #[cfg_attr(feature = "py", pyo3(crate = "reexported::pyo3"))]
pub struct Selection {
	/// Cursor position starting row in buffer.
	pub start_row: i32,
	/// Cursor position starting column in buffer.
	pub start_col: i32,
	/// Cursor position final row in buffer.
	pub end_row: i32,
	/// Cursor position final column in buffer.
	pub end_col: i32,
}

// TODO this re-wrapping of our API is not elegant at all
impl From<codemp_proto::cursor::CursorEvent> for CursorEvent {
	fn from(value: codemp_proto::cursor::CursorEvent) -> Self {
		Self {
			user: value.user,
			cursor: Cursor {
				buffer: value.position.buffer,
				sel: value
					.position
					.cursors
					.into_iter()
					.map(|c| Selection {
						start_row: c.start.row,
						end_row: c.end.row,
						start_col: c.start.col,
						end_col: c.end.col,
					})
					.collect(),
			},
		}
	}
}
