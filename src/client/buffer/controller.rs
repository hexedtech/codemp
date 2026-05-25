//! ### Buffer Controller
//! A [Controller] implementation for buffer actions

use std::sync::Arc;

use tokio::sync::{mpsc, oneshot, watch};

use crate::api::BufferUpdate;
use crate::api::TextChange;
use crate::api::controller::{AsyncReceiver, AsyncSender, Controller, ControllerCallback};
use crate::errors::ControllerResult;
use crate::ext::IgnorableError;

/// A [Controller] to asynchronously interact with remote buffers.
///
/// Each buffer controller internally tracks the last acknowledged state, remaining always in sync
/// with the server while allowing to procedurally receive changes while still sending new ones.
#[derive(Debug)]
#[cfg_attr(feature = "py", pyo3::pyclass(from_py_object))]
#[cfg_attr(feature = "js", napi_derive::napi)]
pub struct BufferController<T: crate::api::CRDT>(pub(crate) Arc<BufferControllerInner<T>>);

// TODO why doesn't derive macro just work?
impl<T: crate::api::CRDT> Clone for BufferController<T> {
	fn clone(&self) -> Self {
		BufferController(self.0.clone())
	}
}

impl<T: crate::api::CRDT> BufferController<T> {
	/// Get id of workspace containing this controller.
	pub fn workspace_id(&self) -> &crate::proto::session::WorkspaceIdentifier {
		&self.0.workspace_id
	}

	/// Get the buffer path.
	pub fn path(&self) -> &str {
		&self.0.path
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
		//let version = version
			//.into_iter()
			//.map(|x| usize::from_ne_bytes(x.to_ne_bytes()))
			//.collect();
		self.0
			.ack_tx
			.send(todo!())
			.unwrap_or_warn("no worker to receive sent ack");
	}
}

#[derive(Debug)]
pub(crate) struct BufferControllerInner<T: crate::api::CRDT> {
	pub(crate) path: String,
	pub(crate) latest_version: watch::Receiver<T::Version>,
	pub(crate) local_version: watch::Receiver<T::Version>,
	pub(crate) ops_in: mpsc::UnboundedSender<TextChange>,
	pub(crate) poller: mpsc::UnboundedSender<oneshot::Sender<()>>,
	pub(crate) content_request: mpsc::Sender<oneshot::Sender<String>>,
	pub(crate) delta_request: mpsc::Sender<oneshot::Sender<Option<BufferUpdate>>>,
	pub(crate) callback: watch::Sender<Option<ControllerCallback<BufferController<T>>>>,
	pub(crate) ack_tx: mpsc::UnboundedSender<T::Version>,
	pub(crate) workspace_id: crate::proto::session::WorkspaceIdentifier,
}

#[cfg_attr(feature = "async-trait", async_trait::async_trait)]
impl<T: crate::api::CRDT> Controller<TextChange, BufferUpdate> for BufferController<T> {}

impl<T: crate::api::CRDT> AsyncSender<TextChange> for BufferController<T> {
	fn send(&self, op: TextChange) -> ControllerResult<()> {
		self.0.ops_in.send(op)?;
		Ok(())
	}
}

#[cfg_attr(feature = "async-trait", async_trait::async_trait)]
impl<T: crate::api::CRDT> AsyncReceiver<BufferUpdate> for BufferController<T> {
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
		self.0.delta_request.send(tx).await?;
		Ok(rx.await?)
	}

	fn callback(&self, cb: impl Into<ControllerCallback<BufferController<T>>>) {
		self.0.callback.send_replace(Some(cb.into()));
	}

	fn clear_callback(&self) {
		if self.0.callback.send(None).is_err() {
			tracing::warn!("no active buffer worker to clear callback");
		}
	}
}
