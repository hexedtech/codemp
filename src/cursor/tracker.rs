use std::sync::Arc;

use tokio::sync::{mpsc, broadcast::{self, error::RecvError}, Mutex};
use tonic::{Streaming, transport::Channel};

use crate::{proto::{Position, Cursor, buffer_client::BufferClient}, errors::IgnorableError, CodempError};

pub struct CursorTracker {
	uid: String,
	op: mpsc::Sender<Cursor>,
	stream: Mutex<broadcast::Receiver<Cursor>>,
}

impl CursorTracker {
	pub async fn moved(&self, path: &str, start: Position, end: Position) -> Result<(), CodempError> {
		Ok(self.op.send(Cursor {
			user: self.uid.clone(),
			buffer: path.to_string(),
			start: start.into(),
			end: end.into(),
		}).await?)
	}

	// TODO is this cancelable? so it can be used in tokio::select!
	// TODO is the result type overkill? should be an option?
	pub async fn recv(&self) -> Result<Cursor, CodempError> {
		let mut stream = self.stream.lock().await;
		match stream.recv().await {
			Ok(x) => Ok(x),
			Err(RecvError::Closed) => Err(CodempError::Channel { send: false }),
			Err(RecvError::Lagged(n)) => {
				tracing::error!("cursor channel lagged behind, skipping {} events", n);
				Ok(stream.recv().await.expect("could not receive after lagging"))
			}
		}
	}

	// fn try_poll(&self) -> Option<Option<Cursor>> {
	// 	match self.stream.try_lock() {
	// 		Err(_) => None,
	// 		Ok(mut x) => match x.try_recv() {
	// 			Ok(x) => Some(Some(x)),
	// 			Err(TryRecvError::Empty) => None,
	// 			Err(TryRecvError::Closed) => Some(None),
	// 			Err(TryRecvError::Lagged(n)) => {
	// 				tracing::error!("cursor channel lagged behind, skipping {} events", n);
	// 				Some(Some(x.try_recv().expect("could not receive after lagging")))
	// 			}
	// 		}
	// 	}
	// }
}

pub(crate) struct CursorTrackerWorker {
	uid: String,
	producer: mpsc::Sender<Cursor>,
	op: mpsc::Receiver<Cursor>,
	channel: Arc<broadcast::Sender<Cursor>>,
}

impl CursorTrackerWorker {
	pub(crate) fn new(uid: String) -> Self {
		let (op_tx, op_rx) = mpsc::channel(64);
		let (cur_tx, _cur_rx) = broadcast::channel(64);
		Self {
			uid,
			producer: op_tx,
			op: op_rx,
			channel: Arc::new(cur_tx),
		}
	}

	pub(crate) fn subscribe(&self) -> CursorTracker {
		CursorTracker {
			uid: self.uid.clone(),
			op: self.producer.clone(),
			stream: Mutex::new(self.channel.subscribe()),
		}
	}

	// TODO is it possible to avoid passing directly tonic Streaming and proto BufferClient ?
	pub(crate) async fn work(mut self, mut rx: Streaming<Cursor>, mut tx: BufferClient<Channel>) {
		loop {
			tokio::select!{
				Ok(Some(cur)) = rx.message() => self.channel.send(cur).unwrap_or_warn("could not broadcast event"),
				Some(op) = self.op.recv() => { tx.moved(op).await.unwrap_or_warn("could not update cursor"); },
				else => break,
			}
		}
	}
}

