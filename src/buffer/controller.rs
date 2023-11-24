//! ### controller
//! 
//! a controller implementation for buffer actions


use std::sync::Arc;

use tokio::sync::oneshot;
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
	poller: mpsc::Sender<oneshot::Sender<()>>,
	_stop: Arc<StopOnDrop>, // just exist
}

impl BufferController {
	pub(crate) fn new(
		name: String,
		content: watch::Receiver<String>,
		operations: mpsc::UnboundedSender<TextChange>,
		poller: mpsc::Sender<oneshot::Sender<()>>,
		stop: mpsc::UnboundedSender<()>,
	) -> Self {
		BufferController {
			name,
			content, operations, poller,
			_stop: Arc::new(StopOnDrop(stop)),
			seen: Arc::new(RwLock::new("".into())),
		}
	}

	pub fn content(&self) -> String {
		self.content.borrow().clone()
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
		let (tx, rx) = oneshot::channel::<()>();
		self.poller.send(tx).await?;
		Ok(rx.await.map_err(|_| crate::Error::Channel { send: false })?)
	}

	fn try_recv(&self) -> crate::Result<Option<TextChange>> {
		let seen = match self.seen.try_read() {
			Err(_) => return Err(crate::Error::Deadlocked),
			Ok(x) => x.clone(),
		};
		let actual = self.content.borrow().clone();
		if seen == actual {
			return Ok(None);
		}
		let change = TextChange::from_diff(&seen, &actual);
		match self.seen.try_write() {
			Err(_) => return Err(crate::Error::Deadlocked),
			Ok(mut w) => *w = actual,
		};
		Ok(Some(change))
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
