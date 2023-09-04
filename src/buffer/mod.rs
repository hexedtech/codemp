//! ### buffer
//!
//! ![demo gif of early buffer sync in action](https://cdn.alemi.dev/codemp/demo-vscode.gif)
//! 
//! a buffer is a container fo text edited by users.
//! this module contains buffer-related operations and helpers to create Operation Sequences
//! (the underlying chunks of changes sent over the wire)

use std::{ops::Range, sync::Arc};

pub(crate) mod worker;

/// buffer controller implementation
pub mod controller;

/// operation factory, with helper functions to produce opseqs
pub mod factory;

pub use factory::OperationFactory;
pub use controller::BufferController as Controller;

use crate::proto::RowCol;


/// an editor-friendly representation of a text change in a buffer
///
/// TODO move in proto
#[derive(Clone, Debug, Default)]
pub struct TextChange {
	/// range of text change, as byte indexes in buffer
	pub span: Range<usize>,
	/// content of text change, as string
	pub content: String,
	/// reference to previous content of buffer
	pub before: Arc<String>,
	/// reference to current content of buffer
	pub after: Arc<String>,
}

impl TextChange {
	/// convert from byte index to row and column.
	/// if `end` is true, span end will be used, otherwise span start
	/// if `after` is true, buffer after change will be used, otherwise buffer before change
	fn index_to_rowcol(&self, end: bool, after: bool) -> RowCol {
		let txt = if after { &self.after } else { &self.before };
		let index = if end { self.span.end } else { self.span.start };
		let row = txt[..index].matches('\n').count() as i32;
		let col = txt[..index].split('\n').last().unwrap_or("").len() as i32;
		RowCol { row, col }
	}

	/// retrn row and column of text change start
	pub fn start(&self) -> RowCol {
		self.index_to_rowcol(false, false)
	}

	/// return row and column of text change end
	pub fn end(&self) -> RowCol {
		self.index_to_rowcol(true, false)
	}
}
