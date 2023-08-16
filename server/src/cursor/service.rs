use std::pin::Pin;

use tokio::sync::{mpsc, broadcast};
use tonic::{Request, Response, Status};

use tokio_stream::{Stream, wrappers::ReceiverStream}; // TODO example used this?

use codemp::proto::{cursor_server::Cursor, UserIdentity, CursorPosition, MovedResponse};
use tracing::info;

type CursorStream = Pin<Box<dyn Stream<Item = Result<CursorPosition, Status>> + Send>>;

pub struct CursorService {
	cursor: broadcast::Sender<CursorPosition>,
}

#[tonic::async_trait]
impl Cursor for CursorService {
	type ListenStream = CursorStream;

	async fn listen(&self, req: Request<UserIdentity>) -> Result<Response<CursorStream>, Status> {
		let mut sub = self.cursor.subscribe();
		let myself = req.into_inner().id;
		let (tx, rx) = mpsc::channel(128);
		tokio::spawn(async move {
			while let Ok(v) = sub.recv().await {
				if v.user == myself { continue }
				tx.send(Ok(v)).await.unwrap(); // TODO unnecessary channel?
			}
		});
		let output_stream = ReceiverStream::new(rx);
		info!("registered new subscriber to cursor updates");
		Ok(Response::new(Box::pin(output_stream)))
	}

	async fn moved(&self, req:Request<CursorPosition>) -> Result<Response<MovedResponse>, Status> {
		match self.cursor.send(req.into_inner()) {
			Ok(_) => Ok(Response::new(MovedResponse { })),
			Err(e) => Err(Status::internal(format!("could not broadcast cursor update: {}", e))),
		}
	}
}

impl Default for CursorService {
	fn default() -> Self {
		let (cur_tx, _cur_rx) = broadcast::channel(64); // TODO hardcoded capacity
		// TODO don't drop receiver because sending event when there are no receivers throws an error
		CursorService {
			cursor: cur_tx,
		}
	}
}
