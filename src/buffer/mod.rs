//! ### buffer
//!
//! ![demo gif of early buffer sync in action](https://cdn.alemi.dev/codemp/demo-vscode.gif)
//! 
//! a buffer is a container fo text edited by users.
//! this module contains buffer-related operations and helpers to create Operation Sequences
//! (the underlying chunks of changes sent over the wire)

use std::ops::Range;

pub(crate) mod worker;

/// buffer controller implementation
pub mod controller;

/// operation factory, with helper functions to produce opseqs
pub mod factory;

pub use factory::OperationFactory;
pub use controller::BufferController as Controller;


/// an editor-friendly representation of a text change in a buffer
///
/// TODO move in proto
#[derive(Debug, Default)]
pub struct TextChange {
	/// range of text change, as byte indexes in buffer
	pub span: Range<usize>,
	/// content of text change, as string
	pub content: String,
}
