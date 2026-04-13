//! ### Cursor
//! Each user in a [crate::Workspace] holds a cursor and can move it across multiple buffers.
//! A cursor spans zero or more characters across one or more lines.

/// cursor worker implementation
pub(crate) mod worker;

/// cursor controller implementation
pub mod controller;
pub use controller::CursorController as Controller;
