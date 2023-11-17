//! ### controller
//! 
//! a controller implementation for buffer actions


use std::sync::Arc;

use tokio::sync::{watch, mpsc, RwLock};
use tonic::async_trait;

use crate::errors::IgnorableError;
use crate::api::Controller;

use crate::api::TextChange;

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
/// upon dropping this handle will stop the associated worker
#[derive(Debug, Clone)]
pub struct BufferController {
	/// unique identifier of buffer
	pub name: String,
	content: watch::Receiver<String>,
	seen: Arc<RwLock<String>>,
	operations: mpsc::UnboundedSender<TextChange>,
	_stop: Arc<StopOnDrop>, // just exist
}

impl BufferController {
	pub(crate) fn new(
		name: String,
		content: watch::Receiver<String>,
		operations: mpsc::UnboundedSender<TextChange>,
		stop: mpsc::UnboundedSender<()>,
	) -> Self {
		BufferController {
			name,
			content, operations,
			_stop: Arc::new(StopOnDrop(stop)),
			seen: Arc::new(RwLock::new("".into())),
		}
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
impl Controller<TextChange> for BufferController {
	type Input = TextChange;

	async fn poll(&self) -> crate::Result<()> {
		let mut poller = self.content.clone();
		loop {
			poller.changed().await?;
			let seen = self.seen.read().await.clone();
			if *poller.borrow() != seen {
				break
			}
		}
		Ok(())
	}

	fn try_recv(&self) -> crate::Result<Option<TextChange>> {
		match self.seen.try_read() {
			Err(_) => Err(crate::Error::Deadlocked),
			Ok(x) => {
				if *self.content.borrow() != *x {
					match self.seen.try_write() {
						Err(_) => Err(crate::Error::Deadlocked),
						Ok(mut w) => {
							let change = TextChange::from_diff(&w, &self.content.borrow());
							*w = self.content.borrow().clone();
							Ok(Some(change))
						}
					}
				} else {
					Ok(None)
				}
			}
		}
	}

	async fn recv(&self) -> crate::Result<TextChange> {
		self.poll().await?;
		let cur = self.seen.read().await.clone();
		let change = TextChange::from_diff(&cur, &self.content.borrow());
		let mut seen = self.seen.write().await;
		*seen = self.content.borrow().clone();
		Ok(change)
	}

	/// enqueue an opseq for processing
	fn send(&self, op: TextChange) -> crate::Result<()> {
		Ok(self.operations.send(op)?)
	}
}
