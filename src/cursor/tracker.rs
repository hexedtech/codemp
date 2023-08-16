use std::sync::Arc;

use tokio::sync::{mpsc, broadcast::{self, error::RecvError}, Mutex};
use tonic::{Streaming, transport::Channel, async_trait};

use crate::{proto::{CursorPosition, cursor_client::CursorClient, RowColumn}, errors::IgnorableError, CodempError, Controller, ControllerWorker};

pub struct CursorTracker {
	uid: String,
	op: mpsc::Sender<CursorPosition>,
	stream: Mutex<broadcast::Receiver<CursorPosition>>,
}

#[async_trait]
impl Controller<CursorPosition> for CursorTracker {
	type Input = (String, RowColumn, RowColumn);

	async fn send(&self, (buffer, start, end): Self::Input) -> Result<(), CodempError> {
		Ok(self.op.send(CursorPosition {
			user: self.uid.clone(),
			start: Some(start),
			end: Some(end),
			buffer,
		}).await?)
	}

	// TODO is this cancelable? so it can be used in tokio::select!
	// TODO is the result type overkill? should be an option?
	async fn recv(&self) -> Result<CursorPosition, CodempError> {
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

	// fn try_poll(&self) -> Option<Option<CursorPosition>> {
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
	producer: mpsc::Sender<CursorPosition>,
	op: mpsc::Receiver<CursorPosition>,
	channel: Arc<broadcast::Sender<CursorPosition>>,
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
}

#[async_trait]
impl ControllerWorker<CursorPosition> for CursorTrackerWorker {
	type Controller = CursorTracker;
	type Tx = CursorClient<Channel>;
	type Rx = Streaming<CursorPosition>;

	fn subscribe(&self) -> CursorTracker {
		CursorTracker {
			uid: self.uid.clone(),
			op: self.producer.clone(),
			stream: Mutex::new(self.channel.subscribe()),
		}
	}

	async fn work(mut self, mut tx: Self::Tx, mut rx: Self::Rx) {
		loop {
			tokio::select!{
				Ok(Some(cur)) = rx.message() => self.channel.send(cur).unwrap_or_warn("could not broadcast event"),
				Some(op) = self.op.recv() => { tx.moved(op).await.unwrap_or_warn("could not update cursor"); },
				else => break,
			}
		}
	}
}

