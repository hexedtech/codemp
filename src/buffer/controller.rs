//! ### controller
//! 
//! a controller implementation for buffer actions


use std::sync::Arc;

use tokio::sync::{watch, mpsc};
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
/// this controller implements [crate::api::OperationFactory], allowing to produce
/// Operation Sequences easily
///
/// upon dropping this handle will stop the associated worker
#[derive(Debug, Clone)]
pub struct BufferController {
	content: watch::Receiver<String>,
	operations: mpsc::UnboundedSender<TextChange>,
	_stop: Arc<StopOnDrop>, // just exist
}

impl BufferController {
	pub(crate) fn new(
		content: watch::Receiver<String>,
		operations: mpsc::UnboundedSender<TextChange>,
		stop: mpsc::UnboundedSender<()>,
	) -> Self {
		BufferController { content, operations, _stop: Arc::new(StopOnDrop(stop)) }
	}
}

#[derive(Debug)]
struct StopOnDrop(mpsc::UnboundedSender<()>);

impl Drop for StopOnDrop {
	fn drop(&mut self) {
		self.0.send(()).unwrap_or_warn("could not send stop message to worker");
	}
}

#[async_trait]
impl Controller<String> for BufferController {
	type Input = TextChange;

	async fn poll(&self) -> Result<(), Error> {
		Ok(self.content.clone().changed().await?)
	}

	fn try_recv(&self) -> Result<Option<String>, Error> {
		Ok(Some(self.content.borrow().clone()))
	}

	async fn recv(&self) -> Result<String, Error> {
		Ok(self.content.borrow().clone())
	}

	/// enqueue an opseq for processing
	fn send(&self, op: TextChange) -> Result<(), Error> {
		Ok(self.operations.send(op)?)
	}
}
