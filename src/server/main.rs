use std::{collections::HashMap, pin::Pin, sync::Arc};

use state::AlterState;
use tonic::{transport::Server, Request, Response, Status};

pub mod proto {
	tonic::include_proto!("workspace");
	tonic::include_proto!("buffer");
}

use tokio::sync::{mpsc, watch};
use tokio_stream::{wrappers::ReceiverStream, Stream, StreamExt}; // TODO example used this?

use proto::buffer_server::{Buffer, BufferServer};
use proto::workspace_server::{Workspace, WorkspaceServer};
use proto::{Operation, SessionRequest, SessionResponse};

use tonic::Streaming;
//use futures::{Stream, StreamExt};

use crate::workspace::Workspace as WorkspaceInstance; // TODO fuck!

pub mod state;
pub mod workspace;

type OperationStream = Pin<Box<dyn Stream<Item = Result<Operation, Status>> + Send>>;

pub struct BufferService {}

#[tonic::async_trait]
impl Buffer for BufferService {
	// type ServerStreamingEchoStream = ResponseStream;
	type AttachStream = OperationStream;

	async fn attach(
		&self,
		req: Request<Streaming<Operation>>,
	) -> Result<tonic::Response<OperationStream>, Status> {
		println!("EchoServer::bidirectional_streaming_echo");

		let mut in_stream = req.into_inner();
		let (tx_og, rx) = mpsc::channel(128);

		// this spawn here is required if you want to handle connection error.
		// If we just map `in_stream` and write it back as `out_stream` the `out_stream`
		// will be drooped when connection error occurs and error will never be propagated
		// to mapped version of `in_stream`.
		let tx = tx_og.clone();
		tokio::spawn(async move {
			while let Some(result) = in_stream.next().await {
				match result {
					Ok(v) => tx
						.send(Ok(Operation {
							action: 1,
							row: 0,
							column: 0,
							op_id: 0,
							text: None,
						}))
						.await
						.expect("working rx"),
					Err(err) => {
						// if let Some(io_err) = match_for_io_error(&err) {
						// 	if io_err.kind() == ErrorKind::BrokenPipe {
						// 		// here you can handle special case when client
						// 		// disconnected in unexpected way
						// 		eprintln!("\tclient disconnected: broken pipe");
						// 		break;
						// 	}
						// }
						eprintln!("Error receiving operation from client");

						match tx.send(Err(err)).await {
							Ok(_) => (),
							Err(_err) => break, // response was droped
						}
					}
				}
			}
			println!("\tstream ended");
		});

		// echo just write the same data that was received
		let out_stream = ReceiverStream::new(rx);

		Ok(Response::new(Box::pin(out_stream) as Self::AttachStream))
	}
}

#[derive(Debug)]
pub struct WorkspaceService {
	tx: mpsc::Sender<AlterState>,
	rx: watch::Receiver<HashMap<String, Arc<WorkspaceInstance>>>,
}

#[tonic::async_trait]
impl Workspace for WorkspaceService {
	async fn create(
		&self,
		request: Request<SessionRequest>,
	) -> Result<Response<SessionResponse>, Status> {
		println!("Got a request: {:?}", request);
		let r = request.into_inner();

		// let w = WorkspaceInstance::new(r.session_key.clone(), r.content.unwrap_or("".to_string()));

		let reply = proto::SessionResponse {
			session_key: r.session_key.clone(),
			accepted: true,
			content: None, // Some(w.content.clone()),
			hash: None,
		};

		// self.tx.send(AlterState::ADD{key: r.session_key.clone(), w}).await.unwrap();

		Ok(Response::new(reply))
	}

	async fn sync(
		&self,
		request: Request<SessionRequest>,
	) -> Result<Response<SessionResponse>, Status> {
		println!("Got a request: {:?}", request);
		let r = request.into_inner();

		if let Some(w) = self.rx.borrow().get(&r.session_key) {
			let reply = proto::SessionResponse {
				session_key: r.session_key,
				accepted: true,
				content: Some(w.content.clone()),
				hash: None,
			};

			Ok(Response::new(reply))
		} else {
			Err(Status::out_of_range("fuck you".to_string()))
		}
	}

	// TODO make it do something
	async fn join(
		&self,
		request: Request<SessionRequest>,
	) -> Result<Response<SessionResponse>, Status> {
		println!("Got a request: {:?}", request);

		let reply = proto::SessionResponse {
			session_key: request.into_inner().session_key,
			accepted: true,
			content: None,
			hash: None,
		};

		Ok(Response::new(reply))
	}

	async fn leave(
		&self,
		request: Request<SessionRequest>,
	) -> Result<Response<SessionResponse>, Status> {
		println!("Got a request: {:?}", request);
		let r = request.into_inner();
		let mut removed = false;

		if self.rx.borrow().get(&r.session_key).is_some() {
			self.tx
				.send(AlterState::REMOVE {
					key: r.session_key.clone(),
				})
				.await
				.unwrap();
			removed = true; // TODO this is a lie! Verify it
		}

		let reply = proto::SessionResponse {
			session_key: r.session_key,
			accepted: removed,
			content: None,
			hash: None,
		};

		Ok(Response::new(reply))
	}
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let addr = "[::1]:50051".parse()?;

	let (tx, rx) = state::run_state_manager();
	let greeter = WorkspaceService { tx, rx };
	let processor = BufferService {};

	Server::builder()
		.add_service(WorkspaceServer::new(greeter))
		.add_service(BufferServer::new(processor))
		.serve(addr)
		.await?;

	Ok(())
}
