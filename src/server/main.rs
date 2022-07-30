use std::collections::VecDeque;
use std::{pin::Pin, sync::Arc};

use tokio_stream::wrappers::ReceiverStream;
use tracing::{info, debug, warn, error};

use operational_transform::OperationSeq;
use state::{AlterState, StateManager};
use tonic::{transport::Server, Request, Response, Status};

pub mod proto {
	tonic::include_proto!("workspace");
	tonic::include_proto!("buffer");
}

use tokio::sync::{mpsc, broadcast};
use tokio_stream::{Stream, StreamExt}; // TODO example used this?

use proto::buffer_server::{Buffer, BufferServer};
use proto::workspace_server::{Workspace, WorkspaceServer};
use proto::{Operation, SessionRequest, SessionResponse};

use tonic::Streaming;
use workspace::BufferView;
//use futures::{Stream, StreamExt};

use crate::workspace::Workspace as WorkspaceInstance; // TODO fuck x2!

pub mod state;
pub mod workspace;

type OperationStream = Pin<Box<dyn Stream<Item = Result<Operation, Status>> + Send>>;

pub struct BufferService {
	state: Arc<StateManager>,
}

fn op_seq(o: &Operation) -> OperationSeq { todo!() }
fn op_net(o: &OperationSeq) -> Operation { todo!() }

// async fn buffer_worker(tx: mpsc::Sender<Result<Operation, Status>>, mut rx:Streaming<Operation>, mut rx_core: mpsc::Receiver<Operation>) {
async fn buffer_worker(bv: BufferView, mut client_rx: Streaming<Operation>, tx_client:mpsc::Sender<Result<Operation, Status>>, mut rx_core:broadcast::Receiver<(String, OperationSeq)>) {
	let mut queue : VecDeque<Operation> = VecDeque::new();
	loop {
		tokio::select! {
			client_op = client_rx.next() => {
				if let Some(result) = client_op {
					match result {
						Ok(op) => {
							bv.op(op_seq(&op)).await.unwrap(); // TODO make OpSeq from network Operation pkt!
							queue.push_back(op);
						},
						Err(status) => {
							error!("error receiving op from client: {:?}", status);
							break;
						}
					}
				}
			},

			server_op = rx_core.recv() => {
				if let Ok(oop) = server_op {
					let mut send_op = true;
					for (i, _op) in queue.iter().enumerate() {
						if true { // TODO must compare underlying OperationSeq here! (op.equals(server_op))
							queue.remove(i);
							send_op = false;
							break;
						} else {
							// serv_op.transform(op); // TODO transform OpSeq !
						}
					}
					if send_op {
						tx_client.send(Ok(op_net(&oop.1))).await.unwrap();
					}
				}
			}
		}
	}
}

#[tonic::async_trait]
impl Buffer for BufferService {
	// type ServerStreamingEchoStream = ResponseStream;
	type AttachStream = OperationStream;

	async fn attach(
		&self,
		req: Request<Streaming<Operation>>,
	) -> Result<tonic::Response<OperationStream>, Status> {
		let session_id : String;
		if let Some(sid) = req.metadata().get("session_id") {
			session_id = sid.to_str().unwrap().to_string();
		} else {
			return Err(Status::failed_precondition("Missing metadata key 'session_id'"));
		}

		let path : String;
		if let Some(p) = req.metadata().get("path") {
			path = p.to_str().unwrap().to_string();
		} else {
			return Err(Status::failed_precondition("Missing metadata key 'path'"));
		}
		// TODO make these above nicer? more concise? idk

		if let Some(workspace) = self.state.workspaces.borrow().get(&session_id) {
			let in_stream = req.into_inner();
			let (tx_og, rx) = mpsc::channel::<Result<Operation, Status>>(128);

			let b: BufferView = workspace.buffers.borrow().get(&path).unwrap().clone();
			let w = workspace.clone();
			tokio::spawn(async move { buffer_worker(b, in_stream, tx_og, w.bus.subscribe()).await; });

			// echo just write the same data that was received
			let out_stream = ReceiverStream::new(rx);

			return Ok(Response::new(Box::pin(out_stream) as Self::AttachStream));
		} else {
			return Err(Status::not_found(format!("Norkspace with session_id {}", session_id)));
		}
	}
}

#[derive(Debug)]
pub struct WorkspaceService {
	state: Arc<StateManager>,
}

#[tonic::async_trait]
impl Workspace for WorkspaceService {
	async fn create(
		&self,
		request: Request<SessionRequest>,
	) -> Result<Response<SessionResponse>, Status> {
		debug!("create request: {:?}", request);
		let r = request.into_inner();

		let w = WorkspaceInstance::new(r.session_key.clone());

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
		debug!("sync request: {:?}", request);
		let r = request.into_inner();

		if let Some(w) = self.state.workspaces.borrow().get(&r.session_key) {
			if let Some(buf) = w.buffers.borrow().get(&r.session_key) {
				let reply = proto::SessionResponse {
					session_key: r.session_key,
					accepted: true,
					content: Some(buf.content.borrow().clone()),
					hash: None,
				};

				Ok(Response::new(reply))
			} else {
				Err(Status::out_of_range("fuck you".to_string()))
			}
		} else {
			Err(Status::out_of_range("fuck you".to_string()))
		}
	}

	// TODO make it do something
	async fn join(
		&self,
		request: Request<SessionRequest>,
	) -> Result<Response<SessionResponse>, Status> {
		debug!("join request: {:?}", request);

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
		debug!("leave request: {:?}", request);
		let r = request.into_inner();
		let mut removed = false;

		if self.state.workspaces.borrow().get(&r.session_key).is_some() {
			self.state.op_tx
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
	tracing_subscriber::fmt::init();

	let addr = "[::1]:50051".parse()?;

	let state = Arc::new(StateManager::new());

	let greeter = WorkspaceService { state: state.clone() };
	let processor = BufferService { state: state.clone() };

	info!("Starting server");

	Server::builder()
		.add_service(WorkspaceServer::new(greeter))
		.add_service(BufferServer::new(processor))
		.serve(addr)
		.await?;
/*

fn main() {
    // install global collector configured based on RUST_LOG env var.
    tracing_subscriber::fmt::init();

    let number_of_yaks = 3;
    // this creates a new event, outside of any spans.
    info!(number_of_yaks, "preparing to shave yaks");

    let number_shaved = yak_shave::shave_all(number_of_yaks);
    info!(
        all_yaks_shaved = number_shaved == number_of_yaks,
        "yak shaving completed."
    );
}
*/

	Ok(())
}
