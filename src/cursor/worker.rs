use std::sync::Arc;

use tokio::sync::{mpsc, oneshot, watch};
use tonic::Streaming;
use uuid::Uuid;

use crate::{api::{controller::{ControllerCallback, ControllerWorker}, Cursor, User}, ext::IgnorableError};
use codemp_proto::cursor::{CursorPosition, CursorEvent};

use super::controller::{CursorController, CursorControllerInner};

pub(crate) struct CursorWorker {
	op: mpsc::Receiver<CursorPosition>,
	map: Arc<dashmap::DashMap<Uuid, User>>,
	stream: mpsc::Receiver<oneshot::Sender<Option<Cursor>>>,
	poll: mpsc::UnboundedReceiver<oneshot::Sender<()>>,
	pollers: Vec<oneshot::Sender<()>>,
	store: std::collections::VecDeque<Cursor>,
	stop: mpsc::UnboundedReceiver<()>,
	controller: CursorController,
	callback: watch::Receiver<Option<ControllerCallback<CursorController>>>,
}

impl CursorWorker {
	pub fn new(user_map: Arc<dashmap::DashMap<Uuid, User>>) -> Self {
		// TODO we should tweak the channel buffer size to better propagate backpressure
		let (op_tx, op_rx) = mpsc::channel(64);
		let (stream_tx, stream_rx) = mpsc::channel(1);
		let (end_tx, end_rx) = mpsc::unbounded_channel();
		let (cb_tx, cb_rx) = watch::channel(None);
		let (poll_tx, poll_rx) = mpsc::unbounded_channel();
		let controller = CursorControllerInner {
			op: op_tx,
			stream: stream_tx,
			stop: end_tx,
			callback: cb_tx,
			poll: poll_tx,
		};
		Self {
			op: op_rx,
			map: user_map,
			stream: stream_rx,
			store: std::collections::VecDeque::default(),
			stop: end_rx,
			controller: CursorController(Arc::new(controller)),
			callback: cb_rx,
			poll: poll_rx,
			pollers: Vec::new(),
		}
	}
}

impl ControllerWorker<Cursor> for CursorWorker {
	type Controller = CursorController;
	type Tx = mpsc::Sender<CursorPosition>;
	type Rx = Streaming<CursorEvent>;

	fn controller(&self) -> CursorController {
		self.controller.clone()
	}

	async fn work(mut self, tx: Self::Tx, mut rx: Self::Rx) {
		loop {
			tracing::debug!("cursor worker polling");
			tokio::select!{
				biased;

				// received stop signal
				Some(()) = self.stop.recv() => { break; },

				// new poller
				Some(poller) = self.poll.recv() => self.pollers.push(poller),

				// client moved their cursor
				Some(op) = self.op.recv() => {
					tracing::debug!("received cursor from editor");
					tx.send(op).await.unwrap_or_warn("could not update cursor");
				},

				// server sents us a cursor
				Ok(Some(cur)) = rx.message() => {
					tracing::debug!("received cursor from server");
					let mut cursor = Cursor {
						buffer: cur.position.buffer.path,
						start: (cur.position.start.row, cur.position.start.col),
						end: (cur.position.end.row, cur.position.end.col),
						user: None,
					};
					let user_id = Uuid::from(cur.user);
					if let Some(user) = self.map.get(&user_id) {
						cursor.user = Some(user.name.clone());
					}
					self.store.push_back(cursor);
					for tx in self.pollers.drain(..) {
						tx.send(()).unwrap_or_warn("poller dropped before unblocking");
					}
					if let Some(cb) = self.callback.borrow().as_ref() {
						tracing::debug!("running cursor callback");
						cb.call(self.controller.clone()); // TODO should this run in its own task/thread?
					}
				},

				// client wants to get next cursor event
				Some(tx) = self.stream.recv() => tx.send(self.store.pop_front())
					.unwrap_or_warn("client gave up receiving"),

				else => break,
			}
		}
	}
}
