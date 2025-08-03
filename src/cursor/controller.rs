//! ### Cursor Controller
//! A [Controller] implementation for [crate::api::Cursor] actions in a [crate::Workspace]

use std::sync::Arc;

use tokio::sync::{mpsc, oneshot, watch};

use crate::{
	api::{
		controller::{AsyncReceiver, AsyncSender, ControllerCallback}, cursor::CursorEvent, Controller, Cursor
	},
	errors::ControllerResult,
};
use codemp_proto::cursor::{CursorPosition, CursorUpdate, RowCol};

/// A [Controller] for asynchronously sending and receiving [Cursor] event.
///
/// An unique [CursorController] exists for each active [crate::Workspace].
#[derive(Debug, Clone)]
#[cfg_attr(feature = "py", pyo3::pyclass)]
#[cfg_attr(feature = "js", napi_derive::napi)]
pub struct CursorController(pub(crate) Arc<CursorControllerInner>);

impl CursorController {
	pub fn workspace_id(&self) -> &str {
		&self.0.workspace_id
	}
}

#[derive(Debug)]
pub(crate) struct CursorControllerInner {
	pub(crate) op: mpsc::UnboundedSender<CursorUpdate>,
	pub(crate) stream: mpsc::Sender<oneshot::Sender<Option<CursorEvent>>>,
	pub(crate) poll: mpsc::UnboundedSender<oneshot::Sender<()>>,
	pub(crate) callback: watch::Sender<Option<ControllerCallback<CursorController>>>,
	pub(crate) workspace_id: String,
}

#[cfg_attr(feature = "async-trait", async_trait::async_trait)]
impl Controller<Cursor, CursorEvent> for CursorController {}

#[cfg_attr(feature = "async-trait", async_trait::async_trait)]
impl AsyncSender<Cursor> for CursorController {
	fn send(&self, mut cursor: Cursor) -> ControllerResult<()> {
		for sel in cursor.sel.iter_mut() {
			if sel.start_row > sel.end_row
				|| (sel.start_row == sel.end_row && sel.start_col > sel.end_col)
			{
				std::mem::swap(&mut sel.start_row, &mut sel.end_row);
				std::mem::swap(&mut sel.start_col, &mut sel.end_col);
			}
		}

		Ok(self.0.op.send(CursorUpdate {
			buffer: cursor.buffer,
			cursors: cursor.sel
				.into_iter()
				.map(|x| CursorPosition {
					start: RowCol {
						row: x.start_row,
						col: x.start_col,
					},
					end: RowCol {
						row: x.end_row,
						col: x.end_col,
					}
				})
				.collect()
		})?)
	}
}

#[cfg_attr(feature = "async-trait", async_trait::async_trait)]
impl AsyncReceiver<CursorEvent> for CursorController {
	async fn try_recv(&self) -> ControllerResult<Option<CursorEvent>> {
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
