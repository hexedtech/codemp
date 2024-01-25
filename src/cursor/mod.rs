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

use crate::proto::cursor::{RowCol, CursorPosition};

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

impl RowCol {
	/// create a RowCol and wrap into an Option, to help build protocol packets
	pub fn wrap(row: i32, col: i32) -> Option<RowCol> {
		Some(RowCol { row, col })
	}
}

impl CursorPosition {
	/// extract start position, defaulting to (0,0), to help build protocol packets
	pub fn start(&self) -> RowCol {
		self.start.clone()
	}

	/// extract end position, defaulting to (0,0), to help build protocol packets
	pub fn end(&self) -> RowCol {
		self.end.clone()
	}
}

impl PartialOrd for RowCol {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		match self.row.partial_cmp(&other.row) {
			Some(core::cmp::Ordering::Equal) => {}
			ord => return ord,
		}
		self.col.partial_cmp(&other.col)
	}
}
