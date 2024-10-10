//! ### Buffer Controller
//! A [Controller] implementation for buffer actions

use std::sync::Arc;

use diamond_types::LocalVersion;
use tokio::sync::{mpsc, oneshot, watch};

use crate::api::controller::{AsyncReceiver, AsyncSender, Controller, ControllerCallback};
use crate::api::BufferUpdate;
use crate::api::TextChange;
use crate::errors::ControllerResult;
use crate::ext::IgnorableError;

/// A [Controller] to asynchronously interact with remote buffers.
///
/// Each buffer controller internally tracks the last acknowledged state, remaining always in sync
/// with the server while allowing to procedurally receive changes while still sending new ones.
#[derive(Debug, Clone)]
#[cfg_attr(any(feature = "py", feature = "py-noabi"), pyo3::pyclass)]
#[cfg_attr(feature = "js", napi_derive::napi)]
pub struct BufferController(pub(crate) Arc<BufferControllerInner>);

impl BufferController {
	/// Get the buffer path.
	pub fn path(&self) -> &str {
		&self.0.name
	}

	/// Return buffer whole content, updating internal acknowledgement tracker.
	pub async fn content(&self) -> ControllerResult<String> {
		let (tx, rx) = oneshot::channel();
		self.0.content_request.send(tx).await?;
		let content = rx.await?;
		Ok(content)
	}

	/// Notify CRDT that changes up to the given version have been merged succesfully.
	pub fn ack(&self, version: Vec<i64>) {
		let version = version
			.into_iter()
			.map(|x| usize::from_ne_bytes(x.to_ne_bytes()))
			.collect();
		self.0
			.ack_tx
			.send(version)
			.unwrap_or_warn("no worker to receive sent ack");
	}
}

#[derive(Debug)]
pub(crate) struct BufferControllerInner {
	pub(crate) name: String,
	pub(crate) latest_version: watch::Receiver<diamond_types::LocalVersion>,
	pub(crate) local_version: watch::Receiver<diamond_types::LocalVersion>,
	pub(crate) ops_in: mpsc::UnboundedSender<TextChange>,
	pub(crate) poller: mpsc::UnboundedSender<oneshot::Sender<()>>,
	pub(crate) content_request: mpsc::Sender<oneshot::Sender<String>>,
	pub(crate) delta_request: mpsc::Sender<(LocalVersion, oneshot::Sender<Option<BufferUpdate>>)>,
	pub(crate) callback: watch::Sender<Option<ControllerCallback<BufferController>>>,
	pub(crate) ack_tx: mpsc::UnboundedSender<LocalVersion>,
}

#[cfg_attr(feature = "async-trait", async_trait::async_trait)]
impl Controller<TextChange, BufferUpdate> for BufferController {}

impl AsyncSender<TextChange> for BufferController {
	fn send(&self, op: TextChange) -> ControllerResult<()> {
		self.0.ops_in.send(op)?;
		Ok(())
	}
}

#[cfg_attr(feature = "async-trait", async_trait::async_trait)]
impl AsyncReceiver<BufferUpdate> for BufferController {
	async fn poll(&self) -> ControllerResult<()> {
		if *self.0.local_version.borrow() != *self.0.latest_version.borrow() {
			return Ok(());
		}

		let (tx, rx) = oneshot::channel::<()>();
		self.0.poller.send(tx)?;
		rx.await?;
		Ok(())
	}

	async fn try_recv(&self) -> ControllerResult<Option<BufferUpdate>> {
		let last_update = self.0.local_version.borrow().clone();
		let latest_version = self.0.latest_version.borrow().clone();

		if last_update == latest_version {
			return Ok(None);
		}

		let (tx, rx) = oneshot::channel();
		self.0.delta_request.send((last_update, tx)).await?;
		Ok(rx.await?)
	}

	fn callback(&self, cb: impl Into<ControllerCallback<BufferController>>) {
		self.0.callback.send_replace(Some(cb.into()));
	}

	fn clear_callback(&self) {
		if self.0.callback.send(None).is_err() {
			tracing::warn!("no active buffer worker to clear callback");
		}
	}
}
