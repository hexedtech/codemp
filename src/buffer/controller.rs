//! ### controller
//! 
//! a controller implementation for buffer actions


use operational_transform::OperationSeq;
use tokio::sync::{watch, mpsc, Mutex, oneshot};
use tonic::async_trait;

use crate::errors::IgnorableError;
use crate::{api::Controller, Error};

use super::TextChange;

/// the buffer controller implementation
///
/// this contains
/// * a watch channel which always contains an updated view of the buffer content
/// * a sink to send buffer operations into
/// * a mutexed broadcast receiver for buffer operations
/// * a channel to stop the associated worker
///
/// for each controller a worker exists, managing outgoing and inbound
/// queues, transforming outbound delayed ops and applying remote changes 
/// to the local buffer
///
/// this controller implements [crate::buffer::OperationFactory], allowing to produce
/// Operation Sequences easily
///
/// upon dropping this handle will stop the associated worker
#[derive(Debug)]
pub struct BufferController {
	content: watch::Receiver<String>,
	operations: mpsc::UnboundedSender<OperationSeq>,
	last_op: Mutex<watch::Receiver<()>>,
	stream: mpsc::UnboundedSender<oneshot::Sender<Option<TextChange>>>,
	stop: mpsc::UnboundedSender<()>,
}

impl BufferController {
	pub(crate) fn new(
		content: watch::Receiver<String>,
		operations: mpsc::UnboundedSender<OperationSeq>,
		stream: mpsc::UnboundedSender<oneshot::Sender<Option<TextChange>>>,
		stop: mpsc::UnboundedSender<()>,
		last_op: Mutex<watch::Receiver<()>>,
	) -> Self {
		BufferController {
			last_op, content, operations, stream, stop,
		}
	}

	pub fn content(&self) -> String {
		self.content.borrow().clone()
	}
}

impl Drop for BufferController {
	fn drop(&mut self) {
		self.stop.send(()).unwrap_or_warn("could not send stop message to worker");
	}
}

#[async_trait]
impl Controller<TextChange> for BufferController {
	type Input = OperationSeq;

	async fn poll(&self) -> Result<(), Error> {
		Ok(self.last_op.lock().await.changed().await?)
	}

	fn try_recv(&self) -> Result<Option<TextChange>, Error> {
		let (tx, rx) = oneshot::channel();
		self.stream.send(tx)?;
		rx.blocking_recv()
			.map_err(|_| Error::Channel { send: false })
	}

	async fn recv(&self) -> Result<TextChange, Error> {
		self.poll().await?;
		let (tx, rx) = oneshot::channel();
		self.stream.send(tx)?;
		Ok(
			rx.await
				.map_err(|_| Error::Channel { send: false })?
				.expect("empty channel after polling")
		)
	}

	/// enqueue an opseq for processing
	fn send(&self, op: OperationSeq) -> Result<(), Error> {
		Ok(self.operations.send(op)?)
	}
}
