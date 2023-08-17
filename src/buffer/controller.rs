use operational_transform::OperationSeq;
use tokio::sync::{watch, mpsc, broadcast, Mutex};
use tonic::async_trait;

use crate::{Controller, Error};
use crate::buffer::factory::{leading_noop, tailing_noop, OperationFactory};

use super::TextChange;

pub struct BufferController {
	content: watch::Receiver<String>,
	operations: mpsc::Sender<OperationSeq>,
	stream: Mutex<broadcast::Receiver<OperationSeq>>,
}

impl BufferController {
	pub(crate) fn new(
		content: watch::Receiver<String>,
		operations: mpsc::Sender<OperationSeq>,
		stream: Mutex<broadcast::Receiver<OperationSeq>>,
	) -> Self {
		BufferController { content, operations, stream }
	}
}

#[async_trait]
impl OperationFactory for BufferController {
	fn content(&self) -> String {
		self.content.borrow().clone()
	}
}

#[async_trait]
impl Controller<TextChange> for BufferController {
	type Input = OperationSeq;

	async fn recv(&self) -> Result<TextChange, Error> {
		let op = self.stream.lock().await.recv().await?;
		let after = self.content.borrow().clone();
		let skip = leading_noop(op.ops()) as usize; 
		let before_len = op.base_len();
		let tail = tailing_noop(op.ops()) as usize;
		let span = skip..before_len-tail;
		let content = after[skip..after.len()-tail].to_string();
		Ok(TextChange { span, content })
	}

	async fn send(&self, op: OperationSeq) -> Result<(), Error> {
		Ok(self.operations.send(op).await?)
	}
}
