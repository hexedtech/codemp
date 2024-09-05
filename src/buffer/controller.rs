//! ### Buffer Controller
//! A [Controller] implementation for buffer actions

use std::sync::Arc;

use diamond_types::LocalVersion;
use tokio::sync::{mpsc, oneshot, watch};

use crate::api::controller::{Controller, ControllerCallback};
use crate::api::TextChange;
use crate::errors::ControllerResult;
use crate::ext::InternallyMutable;

use super::worker::DeltaRequest;

/// A [Controller] to asynchronously interact with remote buffers.
///
/// Each buffer controller internally tracks the last acknowledged state, remaining always in sync
/// with the server while allowing to procedurally receive changes while still sending new ones.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "python", pyo3::pyclass)]
#[cfg_attr(feature = "js", napi_derive::napi)]
pub struct BufferController(pub(crate) Arc<BufferControllerInner>);

impl BufferController {
	/// Get the buffer path
	pub fn path(&self) -> &str {
		&self.0.name
	}

	/// Return buffer whole content, updating internal acknowledgement tracker
	pub async fn content(&self) -> ControllerResult<String> {
		let (tx, rx) = oneshot::channel();
		self.0.content_request.send(tx).await?;
		let content = rx.await?;
		self.0
			.last_update
			.set(self.0.latest_version.borrow().clone());
		Ok(content)
	}
}

#[derive(Debug)]
pub(crate) struct BufferControllerInner {
	pub(crate) name: String,
	pub(crate) latest_version: watch::Receiver<diamond_types::LocalVersion>,
	pub(crate) last_update: InternallyMutable<diamond_types::LocalVersion>,
	pub(crate) ops_in: mpsc::UnboundedSender<(TextChange, oneshot::Sender<LocalVersion>)>,
	pub(crate) poller: mpsc::UnboundedSender<oneshot::Sender<()>>,
	pub(crate) stopper: mpsc::UnboundedSender<()>, // just exist
	pub(crate) content_request: mpsc::Sender<oneshot::Sender<String>>,
	pub(crate) delta_request: mpsc::Sender<DeltaRequest>,
	pub(crate) callback: watch::Sender<Option<ControllerCallback<BufferController>>>,
}

#[cfg_attr(feature = "async-trait", async_trait::async_trait)]
impl Controller<TextChange> for BufferController {
	async fn poll(&self) -> ControllerResult<()> {
		if self.0.last_update.get() != *self.0.latest_version.borrow() {
			return Ok(());
		}

		let (tx, rx) = oneshot::channel::<()>();
		self.0.poller.send(tx)?;
		rx.await?;
		Ok(())
	}

	async fn try_recv(&self) -> ControllerResult<Option<TextChange>> {
		let last_update = self.0.last_update.get();
		let latest_version = self.0.latest_version.borrow().clone();

		if last_update == latest_version {
			return Ok(None);
		}

		let (tx, rx) = oneshot::channel();
		self.0.delta_request.send((last_update, tx)).await?;
		let (v, change) = rx.await?;
		self.0.last_update.set(v);
		Ok(change)
	}

	async fn send(&self, op: TextChange) -> ControllerResult<()> {
		// we let the worker do the updating to the last version and send it back.
		let (tx, rx) = oneshot::channel();
		self.0.ops_in.send((op, tx))?;
		self.0.last_update.set(rx.await?);
		Ok(())
	}

	fn callback(&self, cb: impl Into<ControllerCallback<BufferController>>) {
		if self.0.callback.send(Some(cb.into())).is_err() {
			// TODO should we panic? we failed what we were supposed to do
			tracing::error!("no active buffer worker to run registered callback!");
		}
	}

	fn clear_callback(&self) {
		if self.0.callback.send(None).is_err() {
			tracing::warn!("no active buffer worker to clear callback");
		}
	}

	fn stop(&self) -> bool {
		self.0.stopper.send(()).is_ok()
	}
}
