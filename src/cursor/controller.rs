use tokio::sync::{mpsc, broadcast::{self, error::RecvError}, Mutex};
use tonic::async_trait;

use crate::{proto::{CursorPosition, CursorEvent}, CodempError, Controller};

pub struct CursorController {
	uid: String,
	op: mpsc::Sender<CursorEvent>,
	stream: Mutex<broadcast::Receiver<CursorEvent>>,
}

impl CursorController {
	pub(crate) fn new(
		uid: String,
		op: mpsc::Sender<CursorEvent>,
		stream: Mutex<broadcast::Receiver<CursorEvent>>
	) -> Self {
		CursorController { uid, op, stream }
	}
}

#[async_trait]
impl Controller<CursorEvent> for CursorController {
	type Input = CursorPosition;

	async fn send(&self, cursor: CursorPosition) -> Result<(), CodempError> {
		Ok(self.op.send(CursorEvent {
			user: self.uid.clone(),
			position: Some(cursor),
		}).await?)
	}

	// TODO is this cancelable? so it can be used in tokio::select!
	// TODO is the result type overkill? should be an option?
	async fn recv(&self) -> Result<CursorEvent, CodempError> {
		let mut stream = self.stream.lock().await;
		match stream.recv().await {
			Ok(x) => Ok(x),
			Err(RecvError::Closed) => Err(CodempError::Channel { send: false }),
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
