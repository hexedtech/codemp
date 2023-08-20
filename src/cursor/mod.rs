//! ### cursor
//!
//! ![demo gif of early cursor sync in action](https://cdn.alemi.dev/codemp/demo-nvim.gif)
//! 
//! each user holds a cursor, which consists of multiple highlighted region 
//! on a specific buffer

pub(crate) mod worker;

/// cursor controller implementation
pub mod controller;

pub use controller::CursorController as Controller;

use crate::proto::{RowCol, CursorPosition};

impl From::<RowCol> for (i32, i32) {
	fn from(pos: RowCol) -> (i32, i32) {
		(pos.row, pos.col)
	}
}

impl From::<(i32, i32)> for RowCol {
	fn from((row, col): (i32, i32)) -> Self {
		RowCol { row, col }
	}
}

impl CursorPosition {
	/// extract start position, defaulting to (0,0)
	pub fn start(&self) -> RowCol {
		self.start.clone().unwrap_or((0, 0).into())
	}

	/// extract end position, defaulting to (0,0)
	pub fn end(&self) -> RowCol {
		self.end.clone().unwrap_or((0, 0).into())
	}
}
