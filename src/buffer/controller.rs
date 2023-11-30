//! ### controller
//! 
//! a controller implementation for buffer actions


use std::sync::Arc;

use tokio::sync::oneshot;
use tokio::sync::{watch, mpsc};
use tonic::async_trait;

use crate::errors::IgnorableError;
use crate::api::Controller;

use crate::api::TextChange;

/// the buffer controller implementation
///
/// for each controller a worker exists, managing outgoing and inbound
/// queues, transforming outbound delayed ops and applying remote changes 
/// to the local buffer
///
/// upon dropping this handle will stop the associated worker
#[derive(Debug, Clone)]
pub struct BufferController {
	name: String,
	content: watch::Receiver<String>,
	seen: StatusCheck<String>, // internal buffer previous state
	operations: mpsc::UnboundedSender<TextChange>,
	poller: mpsc::UnboundedSender<oneshot::Sender<()>>,
	_stop: Arc<StopOnDrop>, // just exist
}

impl BufferController {
	pub(crate) fn new(
		name: String,
		content: watch::Receiver<String>,
		operations: mpsc::UnboundedSender<TextChange>,
		poller: mpsc::UnboundedSender<oneshot::Sender<()>>,
		stop: mpsc::UnboundedSender<()>,
	) -> Self {
		BufferController {
			name,
			content, operations, poller,
			seen: StatusCheck::default(),
			_stop: Arc::new(StopOnDrop(stop)),
		}
	}

	/// unique identifier of buffer
	pub fn name(&self) -> &str {
		&self.name
	}

	/// return buffer whole content, updating internal buffer previous state
	pub fn content(&self) -> String {
		self.seen.update(self.content.borrow().clone());
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

	/// block until a text change is available
	/// this returns immediately if one is already available
	async fn poll(&self) -> crate::Result<()> {
		if self.seen.check() != *self.content.borrow() {
			return Ok(()); // short circuit: already available!
		}
		let (tx, rx) = oneshot::channel::<()>();
		self.poller.send(tx)?;
		rx.await.map_err(|_| crate::Error::Channel { send: false })?;
		Ok(())
	}

	/// if a text change is available, return it immediately
	fn try_recv(&self) -> crate::Result<Option<TextChange>> {
		let seen = self.seen.check();
		let actual = self.content.borrow().clone();
		if seen == actual {
			return Ok(None);
		}
		let change = TextChange::from_diff(&seen, &actual);
		self.seen.update(actual);
		Ok(Some(change))
	}

	/// block until a new text change is available, and return it
	async fn recv(&self) -> crate::Result<TextChange> {
		self.poll().await?;
		let seen = self.seen.check();
		let actual = self.content.borrow().clone();
		let change = TextChange::from_diff(&seen, &actual);
		self.seen.update(actual);
		Ok(change)
	}

	/// enqueue a text change for processing
	/// this also updates internal buffer previous state
	fn send(&self, op: TextChange) -> crate::Result<()> {
		let before = self.seen.check();
		self.seen.update(op.apply(&before));
		Ok(self.operations.send(op)?)
	}
}

#[derive(Debug, Clone)]
struct StatusCheck<T : Clone> {
	state: watch::Receiver<T>,
	updater: Arc<watch::Sender<T>>,
}

impl<T : Clone + Default> Default for StatusCheck<T> {
	fn default() -> Self {
		let (tx, rx) = watch::channel(T::default());
		StatusCheck { state: rx, updater: Arc::new(tx) }
	}
}

impl<T : Clone> StatusCheck<T> {
	fn update(&self, state: T) -> T {
		self.updater.send_replace(state)
	}

	fn check(&self) -> T {
		self.state.borrow().clone()
	}
}
