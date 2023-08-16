use std::{sync::Arc, collections::VecDeque, ops::Range};

use operational_transform::OperationSeq;
use tokio::sync::{watch, mpsc, broadcast, Mutex};
use tonic::transport::Channel;
use tonic::{async_trait, Streaming};

use crate::proto::{OperationRequest, RawOp};
use crate::proto::buffer_client::BufferClient;
use crate::{ControllerWorker, Controller, CodempError};
use crate::buffer::factory::{leading_noop, tailing_noop, OperationFactory};

pub struct TextChange {
	pub span: Range<usize>,
	pub content: String,
}

pub struct BufferHandle {
	content: watch::Receiver<String>,
	operations: mpsc::Sender<OperationSeq>,
	stream: Mutex<broadcast::Receiver<OperationSeq>>,
}

#[async_trait]
impl OperationFactory for BufferHandle {
	fn content(&self) -> String {
		self.content.borrow().clone()
	}
}

#[async_trait]
impl Controller<TextChange> for BufferHandle {
	type Input = OperationSeq;

	async fn recv(&self) -> Result<TextChange, CodempError> {
		let op = self.stream.lock().await.recv().await?;
		let after = self.content.borrow().clone();
		let skip = leading_noop(op.ops()) as usize; 
		let before_len = op.base_len();
		let tail = tailing_noop(op.ops()) as usize;
		let span = skip..before_len-tail;
		let content = after[skip..after.len()-tail].to_string();
		Ok(TextChange { span, content })
	}

	async fn send(&self, op: OperationSeq) -> Result<(), CodempError> {
		Ok(self.operations.send(op).await?)
	}
}

pub(crate) struct OperationControllerWorker {
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

impl OperationControllerWorker {
	pub fn new(uid: String, buffer: &str, path: &str) -> Self {
		let (txt_tx, txt_rx) = watch::channel(buffer.to_string());
		let (op_tx, op_rx) = mpsc::channel(64);
		let (s_tx, _s_rx) = broadcast::channel(64);
		OperationControllerWorker {
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
impl ControllerWorker<TextChange> for OperationControllerWorker {
	type Controller = BufferHandle;
	type Tx = BufferClient<Channel>;
	type Rx = Streaming<RawOp>;

	fn subscribe(&self) -> BufferHandle {
		BufferHandle {
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
