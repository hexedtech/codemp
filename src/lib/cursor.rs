use std::{collections::HashMap, sync::Mutex};

use crate::proto::CursorMov;

/// Note that this differs from any hashmap in its put method: no &mut!
pub trait CursorStorage {
	fn get(&self, id: &String) -> Option<Cursor>;
	fn put(&self, id: String, val: Cursor);

	fn update(&self, event: CursorMov) -> Option<Cursor> {
		let mut cur = self.get(&event.user)?;
		cur.buffer = event.path;
		cur.start = (event.row, event.col).into();
		self.put(event.user, cur.clone());
		Some(cur)
	}
}

#[derive(Copy, Clone)]
pub struct Position {
	row: i64,
	col: i64,
}

impl From::<(i64, i64)> for Position {
	fn from((row, col): (i64, i64)) -> Self {
		Position { row, col }
	}
}

#[derive(Clone)]
pub struct Cursor {
	buffer: String,
	start: Position,
	end:   Position,
}

pub struct CursorController {
	users: Mutex<HashMap<String, Cursor>>,
}

impl CursorController {
	pub fn new() -> Self {
		CursorController { users: Mutex::new(HashMap::new()) }
	}
}

impl CursorStorage for CursorController {
	fn get(&self, id: &String) -> Option<Cursor> {
		Some(self.users.lock().unwrap().get(id)?.clone())
	}

	fn put(&self, id: String, val: Cursor) {
		self.users.lock().unwrap().insert(id, val);
	}
}
