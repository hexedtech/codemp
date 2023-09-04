use std::{sync::Arc, collections::VecDeque};

use operational_transform::{OperationSeq, OTError};
use tokio::sync::{watch, mpsc, broadcast, Mutex};
use tonic::transport::Channel;
use tonic::{async_trait, Streaming};

use crate::errors::{IgnorableError, IgnorableDefaultableError};
use crate::proto::{OperationRequest, RawOp};
use crate::proto::buffer_client::BufferClient;
use crate::api::ControllerWorker;

use super::TextChange;
use super::controller::BufferController;
use super::factory::{leading_noop, tailing_noop};


pub(crate) struct BufferControllerWorker {
	uid: String,
	pub(crate) content: watch::Sender<String>,
	pub(crate) operations: mpsc::UnboundedReceiver<OperationSeq>,
	pub(crate) stream: Arc<broadcast::Sender<TextChange>>,
	pub(crate) queue: VecDeque<OperationSeq>,
	receiver: watch::Receiver<String>,
	sender: mpsc::UnboundedSender<OperationSeq>,
	buffer: String,
	path: String,
	stop: mpsc::UnboundedReceiver<()>,
	stop_control: mpsc::UnboundedSender<()>,
}

impl BufferControllerWorker {
	pub fn new(uid: String, buffer: &str, path: &str) -> Self {
		let (txt_tx, txt_rx) = watch::channel(buffer.to_string());
		let (op_tx, op_rx) = mpsc::unbounded_channel();
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

	fn update(&mut self, op: OperationSeq) -> Result<TextChange, OTError> {
		let before = Arc::new(self.buffer.clone());
		let res = op.apply(&before)?;
		self.content.send(res.clone())
			.unwrap_or_warn("error showing updated buffer");
		let after = Arc::new(res.clone());
		self.buffer = res;
		let skip = leading_noop(op.ops()) as usize; 
		let before_len = op.base_len();
		let tail = tailing_noop(op.ops()) as usize;
		let span = skip..before_len-tail;
		let content = after[skip..after.len()-tail].to_string();
		Ok(TextChange { span, content, before, after })
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
			tokio::select! {

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
					let change = self.update(out)
						.unwrap_or_warn_default("coult not update with (transformed) remote operation");
					self.stream.send(change)
						.unwrap_or_warn("could not send operation to server");
				},

				Some(op) = self.operations.recv() => {
					self.queue.push_back(op.clone());
					self.update(op)
						.unwrap_or_warn("could not apply enqueued operation to current buffer");
					while let Some(op) = self.queue.get(0) {
						if !send_opseq(&mut tx, self.uid.clone(), self.path.clone(), op.clone()).await { break }
						self.queue.pop_front();
					}
				},

				Some(()) = self.stop.recv() => {
					break;
				}

				else => break

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
		path, hash: "".into(),
		op: Some(RawOp {
			opseq, user: uid,
		}),
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
