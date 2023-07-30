use std::sync::Arc;

use tokio::sync::{mpsc, broadcast::{self, error::{TryRecvError, RecvError}}, Mutex};
use tonic::async_trait;

use crate::{proto::{Position, Cursor}, errors::IgnorableError, ControllerWorker};

#[async_trait]
pub trait CursorSubscriber {
	async fn send(&self, path: &str, start: Position, end: Position);
	async fn poll(&self) -> Option<Cursor>;
	fn try_poll(&self) -> Option<Option<Cursor>>; // TODO fuck this fuck neovim
}

pub struct CursorControllerHandle {
	uid: String,
	op: mpsc::Sender<Cursor>,
	original: Arc<broadcast::Sender<Cursor>>,
	stream: Mutex<broadcast::Receiver<Cursor>>,
}

impl Clone for CursorControllerHandle {
	fn clone(&self) -> Self {
		CursorControllerHandle {
			uid: self.uid.clone(),
			op: self.op.clone(),
			original: self.original.clone(),
			stream: Mutex::new(self.original.subscribe()),
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

	// TODO is this cancelable? so it can be used in tokio::select!
	async fn poll(&self) -> Option<Cursor> {
		let mut stream = self.stream.lock().await;
		match stream.recv().await {
			Ok(x) => Some(x),
			Err(RecvError::Closed) => None,
			Err(RecvError::Lagged(n)) => {
				tracing::error!("cursor channel lagged behind, skipping {} events", n);
				Some(stream.recv().await.expect("could not receive after lagging"))
			}
		}
	}

	fn try_poll(&self) -> Option<Option<Cursor>> {
		match self.stream.try_lock() {
			Err(_) => None,
			Ok(mut x) => match x.try_recv() {
				Ok(x) => Some(Some(x)),
				Err(TryRecvError::Empty) => None,
				Err(TryRecvError::Closed) => Some(None),
				Err(TryRecvError::Lagged(n)) => {
					tracing::error!("cursor channel lagged behind, skipping {} events", n);
					Some(Some(x.try_recv().expect("could not receive after lagging")))
				}
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
			original: self.channel.clone(),
			stream: Mutex::new(self.channel.subscribe()),
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


