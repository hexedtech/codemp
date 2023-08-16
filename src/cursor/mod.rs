pub(crate) mod worker;
pub mod controller;

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
	pub fn start(&self) -> RowCol {
		self.start.clone().unwrap_or((0, 0).into())
	}

	pub fn end(&self) -> RowCol {
		self.end.clone().unwrap_or((0, 0).into())
	}
}
