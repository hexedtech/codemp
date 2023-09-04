//! ### controller
//! 
//! a controller implementation for buffer actions

use operational_transform::OperationSeq;
use tokio::sync::broadcast::error::TryRecvError;
use tokio::sync::{watch, mpsc, broadcast, Mutex};
use tonic::async_trait;

use crate::errors::IgnorableError;
use crate::{api::Controller, Error};
use crate::buffer::factory::OperationFactory;

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
	last_op: Mutex<watch::Receiver<String>>,
	stream: Mutex<broadcast::Receiver<TextChange>>,
	stop: mpsc::UnboundedSender<()>,
}

impl BufferController {
	pub(crate) fn new(
		content: watch::Receiver<String>,
		operations: mpsc::UnboundedSender<OperationSeq>,
		stream: Mutex<broadcast::Receiver<TextChange>>,
		stop: mpsc::UnboundedSender<()>,
	) -> Self {
		BufferController {
			last_op: Mutex::new(content.clone()),
			content, operations, stream, stop,
		}
	}
}

impl Drop for BufferController {
	fn drop(&mut self) {
		self.stop.send(()).unwrap_or_warn("could not send stop message to worker");
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

	async fn poll(&self) -> Result<(), Error> {
		Ok(self.last_op.lock().await.changed().await?)
	}

	fn try_recv(&self) -> Result<Option<TextChange>, Error> {
		match self.stream.blocking_lock().try_recv() {
			Ok(op) => Ok(Some(op)),
			Err(TryRecvError::Empty) => Ok(None),
			Err(TryRecvError::Closed) => Err(Error::Channel { send: false }),
			Err(TryRecvError::Lagged(n)) => {
				tracing::warn!("buffer channel lagged, skipping {} events", n);
				Ok(self.try_recv()?)
			},
		}
	}

	/// receive an operation seq and transform it into a TextChange from buffer content
	async fn recv(&self) -> Result<TextChange, Error> {
		let op = self.stream.lock().await.recv().await?;
		Ok(op)
	}

	/// enqueue an opseq for processing
	fn send(&self, op: OperationSeq) -> Result<(), Error> {
		Ok(self.operations.send(op)?)
	}
}
