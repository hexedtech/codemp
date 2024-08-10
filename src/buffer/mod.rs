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
