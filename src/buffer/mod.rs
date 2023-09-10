//! ### buffer
//!
//! ![demo gif of early buffer sync in action](https://cdn.alemi.dev/codemp/demo-vscode.gif)
//! 
//! a buffer is a container fo text edited by users.
//! this module contains buffer-related operations and helpers to create Operation Sequences
//! (the underlying chunks of changes sent over the wire)

/// buffer controller implementation
pub mod controller;

pub(crate) mod worker;

pub use controller::BufferController as Controller;


/// an editor-friendly representation of a text change in a buffer
///
/// TODO move in proto
#[derive(Clone, Debug, Default)]
pub struct TextChange {
	/// range of text change, as byte indexes in buffer
	pub span: std::ops::Range<usize>,
	/// content of text change, as string
	pub content: String,
	/// content after this text change
	/// note that this field will probably be dropped, don't rely on it
	pub after: String
}
