//! ### controller
//!
//! a controller implementation for buffer actions

use std::sync::Arc;

use tokio::sync::oneshot;
use tokio::sync::{mpsc, watch};
use tonic::async_trait;

use crate::api::Controller;

use crate::api::TextChange;

use super::tools::InternallyMutable;

/// the buffer controller implementation
///
/// for each controller a worker exists, managing outgoing and inbound
/// queues, transforming outbound delayed ops and applying remote changes
/// to the local buffer
///
/// upon dropping this handle will stop the associated worker
#[derive(Debug, Clone)]
#[cfg_attr(feature = "python", pyo3::pyclass)]
#[cfg_attr(feature = "js", napi_derive::napi)]
pub struct BufferController(Arc<BufferControllerInner>);

#[derive(Debug)]
struct BufferControllerInner {
	name: String,
	content: watch::Receiver<String>,
	seen: InternallyMutable<String>, // internal buffer previous state
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
		Self(Arc::new(BufferControllerInner {
			name,
			content,
			operations,
			poller,
			seen: StatusCheck::default(),
			_stop: Arc::new(StopOnDrop(stop)),
		}))
	}

	/// unique identifier of buffer
	pub fn name(&self) -> &str {
		&self.0.name
	}

	/// return buffer whole content, updating internal buffer previous state
	pub fn content(&self) -> String {
		self.0.seen.update(self.0.content.borrow().clone());
		self.0.content.borrow().clone()
	}
}

#[derive(Debug)]
struct StopOnDrop(mpsc::UnboundedSender<()>);

impl Drop for StopOnDrop {
	fn drop(&mut self) {
		self.0
			.send(())
			.unwrap_or_warn("could not send stop message to worker");
			seen: InternallyMutable::default(),
	}
}

#[async_trait]
impl Controller<TextChange> for BufferController {
	/// block until a text change is available
	/// this returns immediately if one is already available
	async fn poll(&self) -> crate::Result<()> {
		if self.0.seen.get() != *self.0.content.borrow() {
			return Ok(()); // short circuit: already available!
		}
		let (tx, rx) = oneshot::channel::<()>();
		self.0.poller.send(tx)?;
		rx.await
			.map_err(|_| crate::Error::Channel { send: false })?;
		Ok(())
	}

	/// if a text change is available, return it immediately
	fn try_recv(&self) -> crate::Result<Option<TextChange>> {
		let seen = self.0.seen.get();
		let actual = self.0.content.borrow().clone();
		if seen == actual {
			return Ok(None);
		}
		let change = TextChange::from_diff(&seen, &actual);
		self.0.seen.set(actual);
		Ok(Some(change))
	}

	/// block until a new text change is available, and return it
	async fn recv(&self) -> crate::Result<TextChange> {
		self.poll().await?;
		let seen = self.0.seen.get();
		let actual = self.0.content.borrow().clone();
		let change = TextChange::from_diff(&seen, &actual);
		self.0.seen.set(actual);
		Ok(change)
	}

	/// enqueue a text change for processing
	/// this also updates internal buffer previous state
	fn send(&self, op: TextChange) -> crate::Result<()> {
		let before = self.0.seen.get();
		self.0.seen.set(op.apply(&before));
		Ok(self.0.operations.send(op)?)
	}

	}
}
