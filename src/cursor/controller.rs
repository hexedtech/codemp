//! ### Cursor Controller
//! A [Controller] implementation for cursor actions in a [crate::Workspace].

use std::sync::Arc;

use tokio::sync::{mpsc, oneshot, watch};

use crate::{
	api::{Controller, controller::{AsyncReceiver, AsyncSender, ControllerCallback}},
	errors::ControllerResult,
	network::AuthedService,
};
use codemp_proto::cursor::{CursorEvent, CursorUpdate, cursor_client::CursorClient};

/// A [Controller] for asynchronously sending and receiving [CursorEvent]s.
///
/// An unique [CursorController] exists for each active [crate::Workspace].
#[derive(Debug, Clone)]
#[cfg_attr(feature = "py", pyo3::pyclass(from_py_object))]
#[cfg_attr(feature = "js", napi_derive::napi)]
pub struct CursorController(pub(crate) Arc<CursorControllerInner>);

impl CursorController {
	/// Get id of workspace containing this controller.
	pub fn workspace_id(&self) -> &crate::proto::session::WorkspaceIdentifier {
		&self.0.workspace_id
	}
}

#[derive(Debug)]
pub(crate) struct CursorControllerInner {
	pub(crate) op: mpsc::UnboundedSender<CursorUpdate>,
	pub(crate) stream: mpsc::Sender<oneshot::Sender<Option<CursorEvent>>>,
	pub(crate) poll: mpsc::UnboundedSender<oneshot::Sender<()>>,
	pub(crate) callback: watch::Sender<Option<ControllerCallback<CursorController>>>,
	pub(crate) workspace_id: crate::proto::session::WorkspaceIdentifier,
	pub(crate) service: CursorClient<AuthedService>,
}

#[cfg_attr(feature = "async-trait", async_trait::async_trait)]
impl Controller<CursorUpdate, CursorEvent> for CursorController {}

#[cfg_attr(feature = "async-trait", async_trait::async_trait)]
impl AsyncSender<CursorUpdate> for CursorController {
	fn send(&self, mut cursor: CursorUpdate) -> ControllerResult<()> {
		for sel in cursor.cursors.iter_mut() {
			if sel.start.row > sel.finish.row
				|| (sel.start.row == sel.finish.row && sel.start.col > sel.finish.col)
			{
				std::mem::swap(&mut sel.start.row, &mut sel.finish.row);
				std::mem::swap(&mut sel.start.col, &mut sel.finish.col);
			}
		}

		Ok(self.0.op.send(cursor)?)
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
