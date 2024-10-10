//! ### Cursor Controller
//! A [Controller] implementation for [crate::api::Cursor] actions in a [crate::Workspace]

use std::sync::Arc;

use tokio::sync::{mpsc, oneshot, watch};

use crate::{
	api::{
		controller::{AsyncReceiver, AsyncSender, ControllerCallback},
		Controller, Cursor, Selection,
	},
	errors::ControllerResult,
};
use codemp_proto::{
	cursor::{CursorPosition, RowCol},
	files::BufferNode,
};

/// A [Controller] for asynchronously sending and receiving [Cursor] event.
///
/// An unique [CursorController] exists for each active [crate::Workspace].
#[derive(Debug, Clone)]
#[cfg_attr(any(feature = "py", feature = "py-noabi"), pyo3::pyclass)]
#[cfg_attr(feature = "js", napi_derive::napi)]
pub struct CursorController(pub(crate) Arc<CursorControllerInner>);

#[derive(Debug)]
pub(crate) struct CursorControllerInner {
	pub(crate) op: mpsc::UnboundedSender<CursorPosition>,
	pub(crate) stream: mpsc::Sender<oneshot::Sender<Option<Cursor>>>,
	pub(crate) poll: mpsc::UnboundedSender<oneshot::Sender<()>>,
	pub(crate) callback: watch::Sender<Option<ControllerCallback<CursorController>>>,
}

#[cfg_attr(feature = "async-trait", async_trait::async_trait)]
impl Controller<Selection, Cursor> for CursorController {}

#[cfg_attr(feature = "async-trait", async_trait::async_trait)]
impl AsyncSender<Selection> for CursorController {
	fn send(&self, mut cursor: Selection) -> ControllerResult<()> {
		if cursor.start_row > cursor.end_row
			|| (cursor.start_row == cursor.end_row && cursor.start_col > cursor.end_col)
		{
			std::mem::swap(&mut cursor.start_row, &mut cursor.end_row);
			std::mem::swap(&mut cursor.start_col, &mut cursor.end_col);
		}

		Ok(self.0.op.send(CursorPosition {
			buffer: BufferNode {
				path: cursor.buffer,
			},
			start: RowCol {
				row: cursor.start_row,
				col: cursor.start_col,
			},
			end: RowCol {
				row: cursor.end_row,
				col: cursor.end_col,
			},
		})?)
	}
}

#[cfg_attr(feature = "async-trait", async_trait::async_trait)]
impl AsyncReceiver<Cursor> for CursorController {
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
}
