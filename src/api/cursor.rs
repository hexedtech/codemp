//! ### Cursor
//! Represents the position of a remote user's cursor.

#[cfg(feature = "py")]
use pyo3::prelude::*;

/// User cursor position in a buffer
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "py", pyclass)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
// #[cfg_attr(feature = "py", pyo3(crate = "reexported::pyo3"))]
pub struct Cursor {
	/// Cursor start position in buffer, as 0-indexed row-column tuple.
	pub start: (i32, i32),
	/// Cursor end position in buffer, as 0-indexed row-column tuple.
	#[cfg_attr(feature = "serialize", serde(alias = "finish"))] // Lua uses `end` as keyword
	pub end: (i32, i32),
	/// Path of buffer this cursor is on.
	pub buffer: String,
	/// User display name, if provided.
	pub user: Option<String>,
}
