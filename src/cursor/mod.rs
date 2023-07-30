pub mod controller;

use crate::proto::{Position, Cursor};

impl From::<Position> for (i32, i32) {
	fn from(pos: Position) -> (i32, i32) {
		(pos.row, pos.col)
	}
}

impl From::<(i32, i32)> for Position {
	fn from((row, col): (i32, i32)) -> Self {
		Position { row, col }
	}
}

impl Cursor {
	pub fn start(&self) -> Position {
		self.start.clone().unwrap_or((0, 0).into())
	}

	pub fn end(&self) -> Position {
		self.end.clone().unwrap_or((0, 0).into())
	}
}
