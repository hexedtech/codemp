//! ### controller
//!
//! a controller implementation for cursor actions
use std::sync::Arc;

use tokio::sync::{broadcast::{self, error::TryRecvError}, mpsc, watch, Mutex};
use tonic::async_trait;

use crate::api::{controller::ControllerCallback, Controller, Cursor};
use codemp_proto::cursor::{CursorEvent, CursorPosition};
/// the cursor controller implementation
///
/// this contains
/// * the unique identifier of current user
/// * a sink to send movements into
/// * a mutex over a stream of inbound cursor events
/// * a channel to stop the associated worker
///
/// for each controller a worker exists , managing outgoing and inbound event queues
///
/// upon dropping this handle will stop the associated worker
#[derive(Debug, Clone)]
#[cfg_attr(feature = "python", pyo3::pyclass)]
#[cfg_attr(feature = "js", napi_derive::napi)]
pub struct CursorController(pub(crate) Arc<CursorControllerInner>);

#[derive(Debug)]
pub(crate) struct CursorControllerInner {
	pub(crate) op: mpsc::Sender<CursorPosition>,
	pub(crate) last_op: Mutex<watch::Receiver<CursorEvent>>,
	pub(crate) stream: Mutex<broadcast::Receiver<CursorEvent>>,
	pub(crate) callback: watch::Sender<Option<ControllerCallback>>,
	pub(crate) stop: mpsc::UnboundedSender<()>,
}

#[async_trait]
impl Controller<Cursor> for CursorController {
	/// enqueue a cursor event to be broadcast to current workspace
	/// will automatically invert cursor start/end if they are inverted
	async fn send(&self, mut cursor: Cursor) -> crate::Result<()> {
		if cursor.start > cursor.end {
			std::mem::swap(&mut cursor.start, &mut cursor.end);
		}
		Ok(self.0.op.send(cursor.into()).await?)
	}

	/// try to receive without blocking, but will still block on stream mutex
	async fn try_recv(&self) -> crate::Result<Option<Cursor>> {
		let mut stream = self.0.stream.lock().await;
		match stream.try_recv() {
			Ok(x) => Ok(Some(x.into())),
			Err(TryRecvError::Empty) => Ok(None),
			Err(TryRecvError::Closed) => Err(crate::Error::Channel { send: false }),
			Err(TryRecvError::Lagged(n)) => {
				tracing::warn!("cursor channel lagged, skipping {} events", n);
				Ok(stream.try_recv().map(|x| x.into()).ok())
			}
		}
	}

	/// await for changed mutex and then next op change
	async fn poll(&self) -> crate::Result<()> {
		Ok(self.0.last_op.lock().await.changed().await?)
	}

	fn callback(&self, cb: ControllerCallback) {
		if self.0.callback.send(Some(cb)).is_err() {
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
