use std::{collections::HashMap, sync::Mutex};

use tokio::sync::broadcast;
use tracing::{error, debug, warn};

use crate::proto::CursorMov;

/// Note that this differs from any hashmap in its put method: no &mut!
pub trait CursorStorage {
	fn get(&self, id: &str) -> Option<Cursor>;
	fn put(&self, id: String, val: Cursor);

	fn update(&self, event: CursorMov) -> Option<Cursor> {
		let mut cur = self.get(&event.user)?;
		cur.buffer = event.path;
		cur.start = (event.row, event.col).into();
		self.put(event.user, cur.clone());
		Some(cur)
	}
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Position {
	pub row: i64,
	pub col: i64,
}

impl From::<(i64, i64)> for Position {
	fn from((row, col): (i64, i64)) -> Self {
		Position { row, col }
	}
}

#[derive(Clone, Debug, Default)]
pub struct Cursor {
	pub buffer: String,
	pub start: Position,
	pub end:   Position,
}

#[derive(Debug)]
pub struct CursorController {
	users: Mutex<HashMap<String, Cursor>>,
	bus: broadcast::Sender<(String, Cursor)>,
	_bus_keepalive: Mutex<broadcast::Receiver<(String, Cursor)>>,
}

impl Default for CursorController {
	fn default() -> Self {
		let (tx, _rx) = broadcast::channel(64);
		CursorController {
			users: Mutex::new(HashMap::new()),
			bus: tx,
			_bus_keepalive: Mutex::new(_rx),
		}
	}
}

impl CursorController {
	pub fn new() -> Self {
		CursorController::default()
	}

	pub fn sub(&self) -> broadcast::Receiver<(String, Cursor)> {
		self.bus.subscribe()
	}
}

impl CursorStorage for CursorController {
	fn update(&self, event: CursorMov) -> Option<Cursor> {
		debug!("processing cursor event: {:?}", event);
		let mut cur = self.get(&event.user).unwrap_or(Cursor::default());
		cur.buffer = event.path;
		cur.start = (event.row, event.col).into();
		cur.end = (event.row, event.col).into();
		self.put(event.user.clone(), cur.clone());
		if let Err(e) = self.bus.send((event.user, cur.clone())) {
			error!("could not broadcast cursor event: {}", e);
		} else { // this is because once there are no receivers, nothing else can be sent
			if let Err(e) = self._bus_keepalive.lock().unwrap().try_recv() {
				warn!("could not consume event: {}", e);
			}
		}
		Some(cur)
	}
	
	fn get(&self, id: &str) -> Option<Cursor> {
		Some(self.users.lock().unwrap().get(id)?.clone())
	}

	fn put(&self, id: String, val: Cursor) {
		self.users.lock().unwrap().insert(id, val);
	}
}
