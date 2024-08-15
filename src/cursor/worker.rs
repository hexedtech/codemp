use std::sync::Arc;

use tokio::sync::{mpsc, broadcast::{self}, Mutex, watch};
use tonic::{Streaming, async_trait};

use crate::{api::{controller::ControllerWorker, Cursor}, errors::IgnorableError};
use codemp_proto::cursor::{CursorPosition, CursorEvent};

use super::controller::{CursorController, CursorControllerInner};

pub(crate) struct CursorWorker {
	op: mpsc::Receiver<CursorPosition>,
	changed: watch::Sender<CursorEvent>,
	channel: broadcast::Sender<CursorEvent>,
	stop: mpsc::UnboundedReceiver<()>,
	controller: CursorController,
}

impl Default for CursorWorker {
	fn default() -> Self {
		let (op_tx, op_rx) = mpsc::channel(8);
		let (cur_tx, _cur_rx) = broadcast::channel(64);
		let (end_tx, end_rx) = mpsc::unbounded_channel();
		let (change_tx, change_rx) = watch::channel(CursorEvent::default());
		let controller = CursorControllerInner {
			op: op_tx,
			last_op: Mutex::new(change_rx),
			stream: Mutex::new(cur_tx.subscribe()),
			stop: end_tx,
		};
		Self {
			op: op_rx,
			changed: change_tx,
			channel: cur_tx,
			stop: end_rx,
			controller: CursorController(Arc::new(controller)),
		}
	}
}

#[async_trait]
impl ControllerWorker<Cursor> for CursorWorker {
	type Controller = CursorController;
	type Tx = mpsc::Sender<CursorPosition>;
	type Rx = Streaming<CursorEvent>;

	fn controller(&self) -> CursorController {
		self.controller.clone()
	}

	async fn work(mut self, tx: Self::Tx, mut rx: Self::Rx) {
		loop {
			tokio::select!{
				biased;
				Some(()) = self.stop.recv() => { break; },
				Some(op) = self.op.recv() => { tx.send(op).await.unwrap_or_warn("could not update cursor"); },
				Ok(Some(cur)) = rx.message() => {
					self.channel.send(cur.clone()).unwrap_or_warn("could not broadcast event");
					self.changed.send(cur).unwrap_or_warn("could not update last event");
				},
				else => break,
			}
		}
	}
}
