//! ### Cursor
//! Represents the position of a remote user's cursor.

#[cfg(feature = "python")]
use pyo3::prelude::*;

/// User cursor position in a buffer
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "python", pyclass)]
// #[cfg_attr(feature = "python", pyo3(crate = "reexported::pyo3"))]
pub struct Cursor {
	/// Cursor start position in buffer, as 0-indexed row-column tuple.
	pub start: (i32, i32),
	/// Cursor end position in buffer, as 0-indexed row-column tuple.
	pub end: (i32, i32),
	/// Path of buffer this cursor is on.
	pub buffer: String,
	/// User display name, if provided.
	pub user: Option<String>,
}
