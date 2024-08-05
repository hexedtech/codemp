//! # Cursor
//!
//! represents the position of an user's cursor, with
//! information about their identity

use codemp_proto as proto;
// use pyo3::prelude::*;
use uuid::Uuid;

/// user cursor position in a buffer
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "python", pyo3::pyclass)]
pub struct Cursor {
	/// range of text change, as char indexes in buffer previous state
	pub start: (i32, i32),
	pub end: (i32, i32),
	pub buffer: String,
	pub user: Option<Uuid>,
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
			user: Uuid::parse_str(&value.user.id).ok(),
		}
	}
}

impl From<Cursor> for proto::cursor::CursorEvent {
	fn from(value: Cursor) -> Self {
		Self {
			user: proto::common::Identity {
				id: value.user.unwrap_or_default().to_string(),
			},
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
