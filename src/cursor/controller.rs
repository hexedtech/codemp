//! ### Cursor Controller
//! A [Controller] implementation for [Cursor] actions in a [crate::Workspace]

use std::sync::Arc;

use tokio::sync::{mpsc, oneshot, watch};

use crate::{api::{controller::ControllerCallback, Controller, Cursor}, errors::ControllerResult};
use codemp_proto::{cursor::{CursorPosition, RowCol}, files::BufferNode};

/// A [Controller] for asynchronously sending and receiving [Cursor] event.
///
/// An unique [CursorController] exists for each active [crate::Workspace].
#[derive(Debug, Clone)]
#[cfg_attr(feature = "python", pyo3::pyclass)]
#[cfg_attr(feature = "js", napi_derive::napi)]
pub struct CursorController(pub(crate) Arc<CursorControllerInner>);

#[derive(Debug)]
pub(crate) struct CursorControllerInner {
	pub(crate) op: mpsc::Sender<CursorPosition>,
	pub(crate) stream: mpsc::Sender<oneshot::Sender<Option<Cursor>>>,
	pub(crate) poll: mpsc::UnboundedSender<oneshot::Sender<()>>,
	pub(crate) callback: watch::Sender<Option<ControllerCallback<CursorController>>>,
	pub(crate) stop: mpsc::UnboundedSender<()>,
}

#[cfg_attr(feature = "async-trait", async_trait::async_trait)]
impl Controller<Cursor> for CursorController {
	async fn send(&self, mut cursor: Cursor) -> ControllerResult<()> {
		if cursor.start > cursor.end {
			std::mem::swap(&mut cursor.start, &mut cursor.end);
		}
		Ok(self.0.op.send(CursorPosition {
			buffer: BufferNode { path: cursor.buffer },
			start: RowCol { row: cursor.start.0, col: cursor.start.1 },
			end: RowCol { row: cursor.end.0, col: cursor.end.1 },
		}).await?)
	}

	async fn try_recv(&self) -> ControllerResult<Option<Cursor>> {
		let (tx, rx) = oneshot::channel();
		self.0.stream.send(tx).await?;
		Ok(rx.await?)
	}

	async fn poll(&self) -> ControllerResult<()> {
		let (tx, rx) = oneshot::channel();
		self.0.poll.send(tx)?;
		rx.await?;
		Ok(())
	}

	fn callback(&self, cb: impl Into<ControllerCallback<CursorController>>) {
		if self.0.callback.send(Some(cb.into())).is_err() {
			// TODO should we panic? we failed what we were supposed to do
			tracing::error!("no active cursor worker to run registered callback!");
		}
	}

	fn clear_callback(&self) {
		if self.0.callback.send(None).is_err() {
			tracing::warn!("no active cursor worker to clear callback");
		}
	}

	fn stop(&self) -> bool {
		self.0.stop.send(()).is_ok()
	}
}
