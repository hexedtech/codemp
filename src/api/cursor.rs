//! ### Cursor
//! Represents the position of a remote user's cursor.

use codemp_proto as proto;
use uuid::Uuid;

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
	pub user: Option<Uuid>, // TODO this should be a string, not the UUID!
}

impl From<proto::cursor::CursorPosition> for Cursor {
	fn from(value: proto::cursor::CursorPosition) -> Self {
		Self {
			start: (value.start.row, value.start.col),
			end: (value.end.row, value.end.col),
			buffer: value.buffer.path,
			user: None,
		}
	}
}

impl From<Cursor> for proto::cursor::CursorPosition {
	fn from(value: Cursor) -> Self {
		Self {
			buffer: proto::files::BufferNode { path: value.buffer },
			start: proto::cursor::RowCol {
				row: value.start.0,
				col: value.start.1,
			},
			end: proto::cursor::RowCol {
				row: value.end.0,
				col: value.end.1,
			},
		}
	}
}

impl From<proto::cursor::CursorEvent> for Cursor {
	fn from(value: proto::cursor::CursorEvent) -> Self {
		Self {
			start: (value.position.start.row, value.position.start.col),
			end: (value.position.end.row, value.position.end.col),
			buffer: value.position.buffer.path,
			user: Some(value.user.uuid()),
		}
	}
}

impl From<Cursor> for proto::cursor::CursorEvent {
	fn from(value: Cursor) -> Self {
		Self {
			user: value.user.unwrap_or_default().into(),
			position: proto::cursor::CursorPosition {
				buffer: proto::files::BufferNode { path: value.buffer },
				start: proto::cursor::RowCol {
					row: value.start.0,
					col: value.start.1,
				},
				end: proto::cursor::RowCol {
					row: value.end.0,
					col: value.end.1,
				},
			},
		}
	}
}
