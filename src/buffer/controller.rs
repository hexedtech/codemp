use std::{sync::Arc, collections::VecDeque, ops::Range};

use operational_transform::OperationSeq;
use tokio::sync::{watch, mpsc, broadcast, Mutex};
use tonic::async_trait;

use crate::ControllerWorker;
use crate::errors::IgnorableError;
use crate::buffer::factory::{leading_noop, tailing_noop, OperationFactory};

pub struct TextChange {
	pub span: Range<usize>,
	pub content: String,
}

#[async_trait]
pub trait OperationControllerSubscriber : OperationFactory {
	async fn poll(&self) -> Option<TextChange>;
	async fn apply(&self, op: OperationSeq);
}


pub struct OperationControllerHandle {
	content: watch::Receiver<String>,
	operations: mpsc::Sender<OperationSeq>,
	original: Arc<broadcast::Sender<OperationSeq>>,
	stream: Mutex<broadcast::Receiver<OperationSeq>>,
}

impl Clone for OperationControllerHandle {
	fn clone(&self) -> Self {
		OperationControllerHandle {
			content: self.content.clone(),
			operations: self.operations.clone(),
			original: self.original.clone(),
			stream: Mutex::new(self.original.subscribe()),
		}
	}
}

#[async_trait]
impl OperationFactory for OperationControllerHandle {
	fn content(&self) -> String {
		self.content.borrow().clone()
	}
}

#[async_trait]
impl OperationControllerSubscriber for OperationControllerHandle {
	async fn poll(&self) -> Option<TextChange> {
		let op = self.stream.lock().await.recv().await.ok()?;
		let after = self.content.borrow().clone();
		let skip = leading_noop(op.ops()) as usize; 
		let before_len = op.base_len();
		let tail = tailing_noop(op.ops()) as usize;
		let span = skip..before_len-tail;
		let content = after[skip..after.len()-tail].to_string();
		Some(TextChange { span, content })
	}

	async fn apply(&self, op: OperationSeq) {
		self.operations.send(op).await
			.unwrap_or_warn("could not apply+send operation")
	}

	// fn subscribe(&self) -> Self {
	// 	OperationControllerHandle {
	// 		content: self.content.clone(),
	// 		operations: self.operations.clone(),
	// 		original: self.original.clone(),
	// 		stream: Arc::new(Mutex::new(self.original.subscribe())),
	// 	}
	// }
}

#[async_trait]
pub(crate) trait OperationControllerEditor {
	async fn edit(&mut self, path: String, op: OperationSeq) -> bool;
	async fn recv(&mut self) -> Option<OperationSeq>;
}

pub(crate) struct OperationControllerWorker<C : OperationControllerEditor> {
	pub(crate) content: watch::Sender<String>,
	pub(crate) operations: mpsc::Receiver<OperationSeq>,
	pub(crate) stream: Arc<broadcast::Sender<OperationSeq>>,
	pub(crate) queue: VecDeque<OperationSeq>,
	receiver: watch::Receiver<String>,
	sender: mpsc::Sender<OperationSeq>,
	client: C,
	buffer: String,
	path: String,
}

#[async_trait]
impl<C : OperationControllerEditor + Send> ControllerWorker<OperationControllerHandle> for OperationControllerWorker<C> {
	fn subscribe(&self) -> OperationControllerHandle {
		OperationControllerHandle {
			content: self.receiver.clone(),
			operations: self.sender.clone(),
			original: self.stream.clone(),
			stream: Mutex::new(self.stream.subscribe()),
		}
	}

	async fn work(mut self) {
		loop {
			let op = tokio::select! {
				Some(operation) = self.client.recv() => {
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
				if !self.client.edit(self.path.clone(), op.clone()).await { break }
				self.queue.pop_front();
			}
		}
	}

}

impl<C : OperationControllerEditor> OperationControllerWorker<C> {
	pub fn new(client: C, buffer: &str, path: &str) -> Self {
		let (txt_tx, txt_rx) = watch::channel(buffer.to_string());
		let (op_tx, op_rx) = mpsc::channel(64);
		let (s_tx, _s_rx) = broadcast::channel(64);
		OperationControllerWorker {
			content: txt_tx,
			operations: op_rx,
			stream: Arc::new(s_tx),
			receiver: txt_rx,
			sender: op_tx,
			queue: VecDeque::new(),
			buffer: buffer.to_string(),
			path: path.to_string(),
			client,
		}
	}
}
