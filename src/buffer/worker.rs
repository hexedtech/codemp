use std::{sync::Arc, collections::VecDeque};

use operational_transform::OperationSeq;
use tokio::sync::{watch, mpsc, broadcast, Mutex};
use tonic::transport::Channel;
use tonic::{async_trait, Streaming};

use crate::errors::IgnorableError;
use crate::proto::{OperationRequest, RawOp};
use crate::proto::buffer_client::BufferClient;
use crate::ControllerWorker;

use super::TextChange;
use super::controller::BufferController;


pub(crate) struct BufferControllerWorker {
	uid: String,
	pub(crate) content: watch::Sender<String>,
	pub(crate) operations: mpsc::Receiver<OperationSeq>,
	pub(crate) stream: Arc<broadcast::Sender<OperationSeq>>,
	pub(crate) queue: VecDeque<OperationSeq>,
	receiver: watch::Receiver<String>,
	sender: mpsc::Sender<OperationSeq>,
	buffer: String,
	path: String,
	stop: mpsc::UnboundedReceiver<()>,
	stop_control: mpsc::UnboundedSender<()>,
}

impl BufferControllerWorker {
	pub fn new(uid: String, buffer: &str, path: &str) -> Self {
		let (txt_tx, txt_rx) = watch::channel(buffer.to_string());
		let (op_tx, op_rx) = mpsc::channel(64);
		let (s_tx, _s_rx) = broadcast::channel(64);
		let (end_tx, end_rx) = mpsc::unbounded_channel();
		BufferControllerWorker {
			uid,
			content: txt_tx,
			operations: op_rx,
			stream: Arc::new(s_tx),
			receiver: txt_rx,
			sender: op_tx,
			queue: VecDeque::new(),
			buffer: buffer.to_string(),
			path: path.to_string(),
			stop: end_rx,
			stop_control: end_tx,
		}
	}
}

#[async_trait]
impl ControllerWorker<TextChange> for BufferControllerWorker {
	type Controller = BufferController;
	type Tx = BufferClient<Channel>;
	type Rx = Streaming<RawOp>;

	fn subscribe(&self) -> BufferController {
		BufferController::new(
			self.receiver.clone(),
			self.sender.clone(),
			Mutex::new(self.stream.subscribe()),
			self.stop_control.clone(),
		)
	}

	async fn work(mut self, mut tx: Self::Tx, mut rx: Self::Rx) {
		loop {
			let op = tokio::select! {
				Some(operation) = recv_opseq(&mut rx) => {
					let mut out = operation;
					for op in self.queue.iter_mut() {
						(*op, out) = match op.transform(&out) {
							Ok((x, y)) => (x, y),
							Err(e) => {
								tracing::warn!("could not transform enqueued operation: {}", e);
								break
							},
						}
					}
					self.stream.send(out.clone()).unwrap_or_warn("could not send operation to server");
					out
				},
				Some(op) = self.operations.recv() => {
					self.queue.push_back(op.clone());
					op
				},
				Some(()) = self.stop.recv() => {
					break;
				}
				else => break
			};
			self.buffer = op.apply(&self.buffer).unwrap_or_else(|e| {
				tracing::error!("could not update buffer string: {}", e);
				self.buffer
			});
			self.content.send(self.buffer.clone()).unwrap_or_warn("error showing updated buffer");

			while let Some(op) = self.queue.get(0) {
				if !send_opseq(&mut tx, self.uid.clone(), self.path.clone(), op.clone()).await { break }
				self.queue.pop_front();
			}
		}
	}
}

async fn send_opseq(tx: &mut BufferClient<Channel>, uid: String, path: String, op: OperationSeq) -> bool {
	let opseq = match serde_json::to_string(&op) {
		Ok(x) => x,
		Err(e) => {
			tracing::warn!("could not serialize opseq: {}", e);
			return false;
		}
	};
	let req = OperationRequest {
		hash: "".into(),
		user: uid,
		opseq, path,
	};
	match tx.edit(req).await {
		Ok(_) => true,
		Err(e) => {
			tracing::error!("error sending edit: {}", e);
			false
		}
	}
}

async fn recv_opseq(rx: &mut Streaming<RawOp>) -> Option<OperationSeq> {
	match rx.message().await {
		Ok(Some(op)) => match serde_json::from_str(&op.opseq) {
			Ok(x) => Some(x),
			Err(e) => {
				tracing::warn!("could not deserialize opseq: {}", e);
				None
			}
		},
		Ok(None) => None,
		Err(e) => {
			tracing::error!("could not receive edit from server: {}", e);
			None
		}
	}
}
