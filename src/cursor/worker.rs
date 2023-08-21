use std::sync::Arc;

use tokio::sync::{mpsc, broadcast::{self}, Mutex, watch};
use tonic::{Streaming, transport::Channel, async_trait};

use crate::{proto::{cursor_client::CursorClient, CursorEvent}, errors::IgnorableError, ControllerWorker};

use super::controller::CursorController;

pub(crate) struct CursorControllerWorker {
	uid: String,
	producer: mpsc::UnboundedSender<CursorEvent>,
	op: mpsc::UnboundedReceiver<CursorEvent>,
	changed: watch::Sender<CursorEvent>,
	last_op: watch::Receiver<CursorEvent>,
	channel: Arc<broadcast::Sender<CursorEvent>>,
	stop: mpsc::UnboundedReceiver<()>,
	stop_control: mpsc::UnboundedSender<()>,
}

impl CursorControllerWorker {
	pub(crate) fn new(uid: String) -> Self {
		let (op_tx, op_rx) = mpsc::unbounded_channel();
		let (cur_tx, _cur_rx) = broadcast::channel(64);
		let (end_tx, end_rx) = mpsc::unbounded_channel();
		let (change_tx, change_rx) = watch::channel(CursorEvent::default());
		Self {
			uid,
			producer: op_tx,
			op: op_rx,
			changed: change_tx,
			last_op: change_rx,
			channel: Arc::new(cur_tx),
			stop: end_rx,
			stop_control: end_tx,
		}
	}
}

#[async_trait]
impl ControllerWorker<CursorEvent> for CursorControllerWorker {
	type Controller = CursorController;
	type Tx = CursorClient<Channel>;
	type Rx = Streaming<CursorEvent>;

	fn subscribe(&self) -> CursorController {
		CursorController::new(
			self.uid.clone(),
			self.producer.clone(),
			Mutex::new(self.last_op.clone()),
			Mutex::new(self.channel.subscribe()),
			self.stop_control.clone(),
		)
	}

	async fn work(mut self, mut tx: Self::Tx, mut rx: Self::Rx) {
		loop {
			tokio::select!{
				Ok(Some(cur)) = rx.message() => {
					self.channel.send(cur.clone()).unwrap_or_warn("could not broadcast event");
					self.changed.send(cur).unwrap_or_warn("could not update last event");
				},
				Some(op) = self.op.recv() => { tx.moved(op).await.unwrap_or_warn("could not update cursor"); },
				Some(()) = self.stop.recv() => { break; },
				else => break,
			}
		}
	}
}

