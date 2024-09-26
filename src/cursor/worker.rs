use std::sync::Arc;

use tokio::sync::{mpsc, oneshot, watch};
use tonic::Streaming;
use uuid::Uuid;

use crate::{api::{controller::ControllerCallback, Cursor, User}, ext::IgnorableError};
use codemp_proto::cursor::{CursorPosition, CursorEvent};

use super::controller::{CursorController, CursorControllerInner};

struct CursorWorker {
	op: mpsc::Receiver<CursorPosition>,
	map: Arc<dashmap::DashMap<Uuid, User>>,
	stream: mpsc::Receiver<oneshot::Sender<Option<Cursor>>>,
	poll: mpsc::UnboundedReceiver<oneshot::Sender<()>>,
	pollers: Vec<oneshot::Sender<()>>,
	store: std::collections::VecDeque<Cursor>,
	controller: std::sync::Weak<CursorControllerInner>,
	callback: watch::Receiver<Option<ControllerCallback<CursorController>>>,
}

impl CursorController {
	pub(crate) fn spawn(user_map: Arc<dashmap::DashMap<Uuid, User>>, tx: mpsc::Sender<CursorPosition>, rx: Streaming<CursorEvent>) -> Self {
		// TODO we should tweak the channel buffer size to better propagate backpressure
		let (op_tx, op_rx) = mpsc::channel(64);
		let (stream_tx, stream_rx) = mpsc::channel(1);
		let (cb_tx, cb_rx) = watch::channel(None);
		let (poll_tx, poll_rx) = mpsc::unbounded_channel();
		let controller = Arc::new(CursorControllerInner {
			op: op_tx,
			stream: stream_tx,
			callback: cb_tx,
			poll: poll_tx,
		});

		let weak = Arc::downgrade(&controller);

		let worker = CursorWorker {
			op: op_rx,
			map: user_map,
			stream: stream_rx,
			store: std::collections::VecDeque::default(),
			controller: weak,
			callback: cb_rx,
			poll: poll_rx,
			pollers: Vec::new(),
		};

		tokio::spawn(async move { CursorController::work(worker, tx, rx).await });

		CursorController(controller)
	}

	async fn work(mut worker: CursorWorker, tx: mpsc::Sender<CursorPosition>, mut rx: Streaming<CursorEvent>) {
		loop {
			tracing::debug!("cursor worker polling");
			if worker.controller.upgrade().is_none() { break }; // clean exit: all controllers dropped
			tokio::select!{
				biased;

				// new poller
				Some(poller) = worker.poll.recv() => worker.pollers.push(poller),

				// client moved their cursor
				Some(op) = worker.op.recv() => {
					tracing::debug!("received cursor from editor");
					tx.send(op).await.unwrap_or_warn("could not update cursor");
				},

				// server sents us a cursor
				Ok(Some(cur)) = rx.message() => match worker.controller.upgrade() {
					None => break, // clean exit, just weird that we got it here
					Some(controller) => {
						tracing::debug!("received cursor from server");
						let mut cursor = Cursor {
							buffer: cur.position.buffer.path,
							start: (cur.position.start.row, cur.position.start.col),
							end: (cur.position.end.row, cur.position.end.col),
							user: None,
						};
						let user_id = Uuid::from(cur.user);
						if let Some(user) = worker.map.get(&user_id) {
							cursor.user = Some(user.name.clone());
						}
						worker.store.push_back(cursor);
						for tx in worker.pollers.drain(..) {
							tx.send(()).unwrap_or_warn("poller dropped before unblocking");
						}
						if let Some(cb) = worker.callback.borrow().as_ref() {
							tracing::debug!("running cursor callback");
							cb.call(CursorController(controller)); // TODO should this run in its own task/thread?
						}
					},
				},

				// client wants to get next cursor event
				Some(tx) = worker.stream.recv() => tx.send(worker.store.pop_front())
					.unwrap_or_warn("client gave up receiving"),

				else => break,
			}
		}
	}
}
