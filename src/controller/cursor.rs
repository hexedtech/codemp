use std::sync::Arc;

use tokio::sync::{mpsc, broadcast};
use tonic::async_trait;

use crate::{proto::{Position, Cursor}, errors::IgnorableError, controller::ControllerWorker};

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
pub(crate) trait CursorEditor {
	async fn moved(&mut self, cursor: Cursor) -> bool;
	async fn recv(&mut self) -> Option<Cursor>;
}

pub(crate) struct CursorControllerWorker<C : CursorEditor> {
	uid: String,
	producer: mpsc::Sender<Cursor>,
	op: mpsc::Receiver<Cursor>,
	channel: Arc<broadcast::Sender<Cursor>>,
	client: C,
}

impl<C : CursorEditor> CursorControllerWorker<C> {
	pub(crate) fn new(uid: String, client: C) -> Self {
		let (op_tx, op_rx) = mpsc::channel(64);
		let (cur_tx, _cur_rx) = broadcast::channel(64);
		CursorControllerWorker {
			uid, client,
			producer: op_tx,
			op: op_rx,
			channel: Arc::new(cur_tx),
		}
	}
}

#[async_trait]
impl<C : CursorEditor + Send> ControllerWorker<CursorControllerHandle> for CursorControllerWorker<C> {
	fn subscribe(&self) -> CursorControllerHandle {
		CursorControllerHandle {
			uid: self.uid.clone(),
			op: self.producer.clone(),
			stream: self.channel.subscribe(),
			original: self.channel.clone(),
		}
	}

	async fn work(mut self) {
		loop {
			tokio::select!{
				Some(cur) = self.client.recv() => self.channel.send(cur).unwrap_or_warn("could not broadcast event"),
				Some(op) = self.op.recv() => { self.client.moved(op).await; },
				else => break,
			}
		}
	}

}


