//! ### Cursor
//! Represents the position of a remote user's cursor.

#[cfg(any(feature = "py", feature = "py-noabi"))]
use pyo3::prelude::*;

/// User cursor position in a buffer
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "js", napi_derive::napi(object))]
#[cfg_attr(any(feature = "py", feature = "py-noabi"), pyclass)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
// #[cfg_attr(feature = "py", pyo3(crate = "reexported::pyo3"))]
pub struct Cursor {
	/// User who sent the cursor.
	pub user: String,
	/// Cursor selection
	pub sel: Selection,
}

/// A cursor selection span, with row-column tuples
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "js", napi_derive::napi(object))]
#[cfg_attr(any(feature = "py", feature = "py-noabi"), pyclass)]
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
	/// Path of buffer this cursor is on.
	pub buffer: String,
}
