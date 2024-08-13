//! ### controller
//!
//! a controller implementation for buffer actions

use std::sync::Arc;

use diamond_types::LocalVersion;
use tokio::sync::{oneshot, mpsc, watch};
use tonic::async_trait;

use crate::api::Controller;

use crate::api::TextChange;

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
	pub async fn content(&self) -> crate::Result<String> {
		let (tx, rx) = oneshot::channel();
		self.0.content_request.send(tx).await?;
		Ok(rx.await?)
	}
}

#[derive(Debug)]
pub(crate) struct BufferControllerInner {
	pub(crate) name: String,
	pub(crate) latest_version: watch::Receiver<diamond_types::LocalVersion>,
	pub(crate) last_update: InternallyMutable<diamond_types::LocalVersion>,
	pub(crate) ops_in: mpsc::UnboundedSender<TextChange>,
	pub(crate) poller: mpsc::UnboundedSender<oneshot::Sender<()>>,
	pub(crate) stopper: mpsc::UnboundedSender<()>, // just exist
	pub(crate) content_request: mpsc::Sender<oneshot::Sender<String>>,
	pub(crate) delta_request: mpsc::Sender<(LocalVersion, oneshot::Sender<(LocalVersion, TextChange)>)>,
}

#[async_trait]
impl Controller<TextChange> for BufferController {
	/// block until a text change is available
	/// this returns immediately if one is already available
	async fn poll(&self) -> crate::Result<()> {
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
	async fn try_recv(&self) -> crate::Result<Option<TextChange>> {
		let last_update = self.0.last_update.get();
		let latest_version = self.0.latest_version.borrow().clone();

		if last_update == latest_version {
			return Ok(None);
		}

		let (tx, rx) = oneshot::channel();
		self.0.delta_request.send((last_update, tx)).await?;
		let (v, change) = rx.await?;
		self.0.last_update.set(v);
		Ok(Some(change))
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
