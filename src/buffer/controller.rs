//! ### controller
//!
//! a controller implementation for buffer actions

use std::sync::Arc;

use diamond_types::LocalVersion;
use tokio::sync::oneshot;
use tokio::sync::{mpsc, watch};
use tonic::async_trait;

use crate::api::Controller;

use crate::api::TextChange;

use crate::api::Op;

use crate::ext::InternallyMutable;

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
pub struct BufferController(pub(crate) Arc<BufferControllerInner>);

impl BufferController {
	/// unique identifier of buffer
	pub fn name(&self) -> &str {
		&self.0.name
	}

	/// return buffer whole content, updating internal buffer previous state
	pub fn content(&self) -> String {
		// this function either needs a ref to the worker
		// or needs to basically mirror all the operations that go through it.
		// yikes.
		todo!() // TODO ouch
	}
}

#[derive(Debug)]
pub(crate) struct BufferControllerInner {
	name: String,
	latest_version: watch::Receiver<diamond_types::LocalVersion>,
	last_update: InternallyMutable<diamond_types::LocalVersion>,
	ops_in: mpsc::UnboundedSender<TextChange>,
	ops_out: mpsc::UnboundedReceiver<(LocalVersion, Option<Op>)>,
	poller: mpsc::UnboundedSender<oneshot::Sender<()>>,
	stopper: mpsc::UnboundedSender<()>, // just exist
}

impl BufferControllerInner {
	pub(crate) fn new(
		name: String,
		latest_version: watch::Receiver<diamond_types::LocalVersion>,
		ops_in: mpsc::UnboundedSender<TextChange>,
		ops_out: mpsc::UnboundedReceiver<(LocalVersion, Option<Op>)>,
		poller: mpsc::UnboundedSender<oneshot::Sender<()>>,
		stopper: mpsc::UnboundedSender<()>,
	) -> Self {
		Self {
			name,
			latest_version,
			last_update: InternallyMutable::new(diamond_types::LocalVersion::default()),
			ops_in,
			ops_out,
			poller,
			stopper,
		}
	}
}

#[async_trait]
impl Controller<TextChange> for BufferController {
	/// block until a text change is available
	/// this returns immediately if one is already available
	async fn poll(&self) -> crate::Result<()> {
		// TODO there might be some extra logic we can play with using `seen` and `not seen` yet
		// mechanics, not just the comparison. nevermind, the `has_changed` etc stuff needs mut self, yuk.

		if self.0.last_update.get() != *self.0.latest_version.borrow() {
			return Ok(());
		}

		let (tx, rx) = oneshot::channel::<()>();
		self.0.poller.send(tx)?;
		rx.await
			.map_err(|_| crate::Error::Channel { send: false })?;
		Ok(())
	}

	/// if a text change is available, return it immediately
	fn try_recv(&self) -> crate::Result<Option<TextChange>> {
		let last_update = self.0.last_update.get();
		let latest_version = self.0.latest_version.borrow().clone();

		if last_update == latest_version {
			return Ok(None);
		}

		if let Ok((lv, Some(op))) = self.0.ops_out.try_recv() {
			// if the current change has
			self.0.last_update.set(lv);
			return Ok(Some(TextChange::from(op)));
		}

		return Err(crate::Error::Channel { send: false });
	}

	/// block until a new text change is available, and return it
	async fn recv(&self) -> crate::Result<TextChange> {
		let last_update = self.0.last_update.get();

		// no need to poll here? as soon as we have new changes we return them!
		if let Some((lv, Some(op))) = self.0.ops_out.recv().await {
			self.0.last_update.set(lv);
			Ok(TextChange::from(op))
		} else {
			Err(crate::Error::Channel { send: false })
		}
	}

	/// enqueue a text change for processing
	/// this also updates internal buffer previous state
	fn send(&self, op: TextChange) -> crate::Result<()> {
		// we let the worker do the updating to the last version and send it back.
		Ok(self.0.ops_in.send(op)?)
	}

	fn stop(&self) -> bool {
		self.0.stopper.send(()).is_ok()
	}
}
