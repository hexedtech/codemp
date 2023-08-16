pub mod tracker;

use crate::proto::{RowColumn, CursorPosition};

impl From::<RowColumn> for (i32, i32) {
	fn from(pos: RowColumn) -> (i32, i32) {
		(pos.row, pos.col)
	}
}

impl From::<(i32, i32)> for RowColumn {
	fn from((row, col): (i32, i32)) -> Self {
		RowColumn { row, col }
	}
}

impl CursorPosition {
	pub fn start(&self) -> RowColumn {
		self.start.clone().unwrap_or((0, 0).into())
	}

	pub fn end(&self) -> RowColumn {
		self.end.clone().unwrap_or((0, 0).into())
	}
}
