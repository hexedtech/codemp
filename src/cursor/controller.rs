//! ### controller
//! 
//! a controller implementation for cursor actions

use tokio::sync::{mpsc, broadcast::{self, error::{TryRecvError, RecvError}}, Mutex, watch};
use tonic::async_trait;

use crate::{api::Controller, errors::IgnorableError, proto::cursor::{CursorEvent, CursorPosition}};

/// the cursor controller implementation
///
/// this contains
/// * the unique identifier of current user
/// * a sink to send movements into
/// * a mutex over a stream of inbound cursor events
/// * a channel to stop the associated worker
///
/// for each controller a worker exists, managing outgoing and inbound event queues
///
/// upon dropping this handle will stop the associated worker
#[derive(Debug)]
pub struct CursorController {
	op: mpsc::UnboundedSender<CursorPosition>,
	last_op: Mutex<watch::Receiver<CursorEvent>>,
	stream: Mutex<broadcast::Receiver<CursorEvent>>,
	stop: mpsc::UnboundedSender<()>,
}

impl Drop for CursorController {
	fn drop(&mut self) {
		self.stop.send(()).unwrap_or_warn("could not stop cursor actor")
	}
}

impl CursorController {
	pub(crate) fn new(
		op: mpsc::UnboundedSender<CursorPosition>,
		last_op: Mutex<watch::Receiver<CursorEvent>>,
		stream: Mutex<broadcast::Receiver<CursorEvent>>,
		stop: mpsc::UnboundedSender<()>,
	) -> Self {
		CursorController { op, last_op, stream, stop }
	}
}

#[async_trait]
impl Controller<CursorEvent> for CursorController {
	type Input = CursorPosition;

	/// enqueue a cursor event to be broadcast to current workspace
	/// will automatically invert cursor start/end if they are inverted
	fn send(&self, mut cursor: CursorPosition) -> crate::Result<()> {
		if cursor.start > cursor.end {
			std::mem::swap(&mut cursor.start, &mut cursor.end);
		}
		Ok(self.op.send(cursor)?)
	}

	/// try to receive without blocking, but will still block on stream mutex
	fn try_recv(&self) -> crate::Result<Option<CursorEvent>> {
		let mut stream = self.stream.blocking_lock();
		match stream.try_recv() {
			Ok(x) => Ok(Some(x)),
			Err(TryRecvError::Empty) => Ok(None),
			Err(TryRecvError::Closed) => Err(crate::Error::Channel { send: false }),
			Err(TryRecvError::Lagged(n)) => {
				tracing::warn!("cursor channel lagged, skipping {} events", n);
				Ok(stream.try_recv().ok())
			},
		}
	}

	// TODO is this cancelable? so it can be used in tokio::select!
	// TODO is the result type overkill? should be an option?
	/// get next cursor event from current workspace, or block until one is available
	async fn recv(&self) -> crate::Result<CursorEvent> {
		let mut stream = self.stream.lock().await;
		match stream.recv().await {
			Ok(x) => Ok(x),
			Err(RecvError::Closed) => Err(crate::Error::Channel { send: false }),
			Err(RecvError::Lagged(n)) => {
				tracing::error!("cursor channel lagged behind, skipping {} events", n);
				Ok(stream.recv().await.expect("could not receive after lagging"))
			}
		}
	}

	/// await for changed mutex and then next op change
	async fn poll(&self) -> crate::Result<()> {
		Ok(self.last_op.lock().await.changed().await?)
	}

}
