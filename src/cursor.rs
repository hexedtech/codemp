use std::sync::Arc;

use tokio::sync::{mpsc, broadcast};
use tonic::async_trait;

use crate::{proto::{Position, Cursor}, errors::IgnorableError};

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

#[async_trait]
pub trait CursorSubscriber {
	async fn send(&self, path: &str, start: Position, end: Position);
	async fn poll(&mut self) -> Option<Cursor>;
}

pub struct CursorControllerHandle {
	uid: String,
	op: mpsc::Sender<Cursor>,
	stream: broadcast::Receiver<Cursor>,
	original: Arc<broadcast::Sender<Cursor>>,
}

impl Clone for CursorControllerHandle {
	fn clone(&self) -> Self {
		Self {
			uid: self.uid.clone(),
			op: self.op.clone(),
			stream: self.original.subscribe(),
			original: self.original.clone()
		}
	}
}

#[async_trait]
impl CursorSubscriber for CursorControllerHandle {
	async fn send(&self, path: &str, start: Position, end: Position) {
		self.op.send(Cursor {
			user: self.uid.clone(),
			buffer: path.to_string(),
			start: Some(start),
			end: Some(end),
		}).await.unwrap_or_warn("could not send cursor op")
	}

	async fn poll(&mut self) -> Option<Cursor> {
		match self.stream.recv().await {
			Ok(x) => Some(x),
			Err(e) => {
				tracing::warn!("could not poll for cursor: {}", e);
				None
			}
		}
	}
}

#[async_trait]
pub(crate) trait CursorProvider<T>
where T : CursorSubscriber {
	fn subscribe(&self) -> T;
	fn broadcast(&self, op: Cursor);
	async fn wait(&mut self) -> Option<Cursor>;
}

pub(crate) struct CursorControllerWorker {
	uid: String,
	producer: mpsc::Sender<Cursor>,
	op: mpsc::Receiver<Cursor>,
	channel: Arc<broadcast::Sender<Cursor>>,
}

impl CursorControllerWorker {
	pub(crate) fn new(uid: String) -> Self {
		let (op_tx, op_rx) = mpsc::channel(64);
		let (cur_tx, _cur_rx) = broadcast::channel(64);
		CursorControllerWorker {
			uid,
			producer: op_tx,
			op: op_rx,
			channel: Arc::new(cur_tx),
		}
	}
}

#[async_trait]
impl CursorProvider<CursorControllerHandle> for CursorControllerWorker {
	fn broadcast(&self, op: Cursor) {
		self.channel.send(op).unwrap_or_warn("could not broadcast cursor event")
	}

	async fn wait(&mut self) -> Option<Cursor> {
		self.op.recv().await
	}

	fn subscribe(&self) -> CursorControllerHandle {
		CursorControllerHandle {
			uid: self.uid.clone(),
			op: self.producer.clone(),
			stream: self.channel.subscribe(),
			original: self.channel.clone(),
		}
	}

}

