use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use tokio::sync::{watch, mpsc, oneshot};
use tonic::{async_trait, Streaming};
use uuid::Uuid;
use woot::crdt::CRDT;
use woot::woot::Woot;

use crate::errors::IgnorableError;
use crate::api::controller::ControllerWorker;
use crate::api::TextChange;
use codemp_proto::buffer::{BufferEvent, Operation};

use super::controller::{BufferController, BufferControllerInner};

pub(crate) struct BufferWorker {
	_user_id: Uuid,
	buffer: Woot,
	content: watch::Sender<String>,
	operations: mpsc::UnboundedReceiver<TextChange>,
	poller: mpsc::UnboundedReceiver<oneshot::Sender<()>>,
	pollers: Vec<oneshot::Sender<()>>,
	stop: mpsc::UnboundedReceiver<()>,
	controller: Arc<BufferControllerInner>,
}

impl BufferWorker {
	pub fn new(user_id: Uuid, path: &str) -> Self {
		let (txt_tx, txt_rx) = watch::channel("".to_string());
		let (op_tx, op_rx) = mpsc::unbounded_channel();
		let (end_tx, end_rx) = mpsc::unbounded_channel();
		let (poller_tx, poller_rx) = mpsc::unbounded_channel();
		let mut hasher = DefaultHasher::new();
		user_id.hash(&mut hasher);
		let site_id = hasher.finish() as usize;
		let controller = BufferControllerInner::new(
			path.to_string(),
			txt_rx,
			op_tx,
			poller_tx,
			end_tx,
		);
		BufferWorker {
			_user_id: user_id,
			buffer: Woot::new(site_id % (2<<10), ""), // TODO remove the modulo, only for debugging!
			content: txt_tx,
			operations: op_rx,
			poller: poller_rx,
			pollers: Vec::new(),
			stop: end_rx,
			controller: Arc::new(controller),
		}
	}
}

#[async_trait]
impl ControllerWorker<TextChange> for BufferWorker {
	type Controller = BufferController;
	type Tx = mpsc::Sender<Operation>;
	type Rx = Streaming<BufferEvent>;

	fn subscribe(&self) -> BufferController {
		BufferController(self.controller.clone())
	}

	async fn work(mut self, tx: Self::Tx, mut rx: Self::Rx) {
		loop {
			// block until one of these is ready
			tokio::select! {
				biased;

				// received stop signal
				_ = self.stop.recv() => break,

				// received a new poller, add it to collection
				res = self.poller.recv() => match res {
					None => break tracing::error!("poller channel closed"),
					Some(tx) => self.pollers.push(tx),
				},

				// received a text change from editor
				res = self.operations.recv() => match res {
					None => break tracing::debug!("stopping: editor closed channel"),
					Some(change) => match change.transform(&self.buffer) {
						Err(e) => break tracing::error!("could not apply operation from client: {}", e),
						Ok(ops) => {
							for op in ops {
								self.buffer.merge(op.0.clone());
								let operation = Operation { 
									data: postcard::to_extend(&op.0, Vec::new()).unwrap(),
								};
								if let Err(e) = tx.send(operation).await {
									tracing::error!("server refused to broadcast {}: {}", op.0, e);
								}
							}
							self.content.send(self.buffer.view())
								.unwrap_or_warn("could not send buffer update");
						},
					}
				},

				// received a message from server
				res = rx.message() => match res {
					Err(_e) => break,
					Ok(None) => break,
					Ok(Some(change)) => match postcard::from_bytes::<woot::crdt::Op>(&change.op.data) {
						Ok(op) => { // TODO here in change we receive info about the author, maybe propagate?
							self.buffer.merge(op);
							self.content.send(self.buffer.view()).unwrap_or_warn("could not send buffer update");
							for tx in self.pollers.drain(..) {
								tx.send(()).unwrap_or_warn("could not wake up poller");
							}
						},
						Err(e) => tracing::error!("could not deserialize operation from server: {}", e),
					},
				},
			}
		}
	}
}
