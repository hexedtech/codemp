use tokio::sync::{mpsc, broadcast::{self, error::RecvError}, Mutex};
use tonic::async_trait;

use crate::{proto::{CursorPosition, CursorEvent}, Error, Controller, errors::IgnorableError};

pub struct CursorController {
	uid: String,
	op: mpsc::Sender<CursorEvent>,
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
		uid: String,
		op: mpsc::Sender<CursorEvent>,
		stream: Mutex<broadcast::Receiver<CursorEvent>>,
		stop: mpsc::UnboundedSender<()>,
	) -> Self {
		CursorController { uid, op, stream, stop }
	}
}

#[async_trait]
impl Controller<CursorEvent> for CursorController {
	type Input = CursorPosition;

	async fn send(&self, cursor: CursorPosition) -> Result<(), Error> {
		Ok(self.op.send(CursorEvent {
			user: self.uid.clone(),
			position: Some(cursor),
		}).await?)
	}

	// TODO is this cancelable? so it can be used in tokio::select!
	// TODO is the result type overkill? should be an option?
	async fn recv(&self) -> Result<CursorEvent, Error> {
		let mut stream = self.stream.lock().await;
		match stream.recv().await {
			Ok(x) => Ok(x),
			Err(RecvError::Closed) => Err(Error::Channel { send: false }),
			Err(RecvError::Lagged(n)) => {
				tracing::error!("cursor channel lagged behind, skipping {} events", n);
				Ok(stream.recv().await.expect("could not receive after lagging"))
			}
		}
	}

	// fn try_poll(&self) -> Option<Option<CursorPosition>> {
	// 	match self.stream.try_lock() {
	// 		Err(_) => None,
	// 		Ok(mut x) => match x.try_recv() {
	// 			Ok(x) => Some(Some(x)),
	// 			Err(TryRecvError::Empty) => None,
	// 			Err(TryRecvError::Closed) => Some(None),
	// 			Err(TryRecvError::Lagged(n)) => {
	// 				tracing::error!("cursor channel lagged behind, skipping {} events", n);
	// 				Some(Some(x.try_recv().expect("could not receive after lagging")))
	// 			}
	// 		}
	// 	}
	// }
}
