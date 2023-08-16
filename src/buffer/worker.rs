use std::{sync::Arc, collections::VecDeque};

use operational_transform::OperationSeq;
use tokio::sync::{watch, mpsc, broadcast, Mutex};
use tonic::transport::Channel;
use tonic::{async_trait, Streaming};

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
}

impl BufferControllerWorker {
	pub fn new(uid: String, buffer: &str, path: &str) -> Self {
		let (txt_tx, txt_rx) = watch::channel(buffer.to_string());
		let (op_tx, op_rx) = mpsc::channel(64);
		let (s_tx, _s_rx) = broadcast::channel(64);
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
		}
	}
}

#[async_trait]
impl ControllerWorker<TextChange> for BufferControllerWorker {
	type Controller = BufferController;
	type Tx = BufferClient<Channel>;
	type Rx = Streaming<RawOp>;

	fn subscribe(&self) -> BufferController {
		BufferController {
			content: self.receiver.clone(),
			operations: self.sender.clone(),
			stream: Mutex::new(self.stream.subscribe()),
		}
	}

	async fn work(mut self, mut tx: Self::Tx, mut rx: Self::Rx) {
		loop {
			let op = tokio::select! {
				Some(operation) = recv_opseq(&mut rx) => {
					let mut out = operation;
					for op in self.queue.iter_mut() {
						(*op, out) = op.transform(&out).unwrap();
					}
					self.stream.send(out.clone()).unwrap();
					out
				},
				Some(op) = self.operations.recv() => {
					self.queue.push_back(op.clone());
					op
				},
				else => break
			};
			self.buffer = op.apply(&self.buffer).unwrap();
			self.content.send(self.buffer.clone()).unwrap();

			while let Some(op) = self.queue.get(0) {
				if !send_opseq(&mut tx, self.uid.clone(), self.path.clone(), op.clone()).await { break }
				self.queue.pop_front();
			}
		}
	}
}

async fn send_opseq(tx: &mut BufferClient<Channel>, uid: String, path: String, op: OperationSeq) -> bool {
	let req = OperationRequest {
		hash: "".into(),
		opseq: serde_json::to_string(&op).unwrap(),
		path,
		user: uid,
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
		Ok(Some(op)) => Some(serde_json::from_str(&op.opseq).unwrap()),
		Ok(None) => None,
		Err(e) => {
			tracing::error!("could not receive edit from server: {}", e);
			None
		}
	}
}
