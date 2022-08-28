use std::collections::VecDeque;
use std::{pin::Pin, sync::Arc};

use tokio_stream::wrappers::ReceiverStream;
use tracing::error;

use operational_transform::OperationSeq;
use tonic::{Request, Response, Status};

pub mod proto {
	tonic::include_proto!("session");
	tonic::include_proto!("workspace");
	tonic::include_proto!("buffer");
}

use tokio::sync::{broadcast, mpsc};
use tokio_stream::{Stream, StreamExt}; // TODO example used this?

use proto::buffer_server::{Buffer, BufferServer};
use proto::Operation;

use tonic::Streaming;
//use futures::{Stream, StreamExt};

use crate::actor::{buffer::BufferView, state::StateManager};
use crate::events::Event;

use self::proto::{BufferPayload, BufferResponse}; // TODO fuck x2!

type OperationStream = Pin<Box<dyn Stream<Item = Result<Operation, Status>> + Send>>;

pub struct BufferService {
	state: Arc<StateManager>,
}

fn op_seq(_o: &Operation) -> OperationSeq {
	todo!()
}
fn op_net(_o: &OperationSeq) -> Operation {
	todo!()
}

// async fn buffer_worker(tx: mpsc::Sender<Result<Operation, Status>>, mut rx:Streaming<Operation>, mut rx_core: mpsc::Receiver<Operation>) {
async fn buffer_worker(
	bv: BufferView,
	mut client_rx: Streaming<Operation>,
	tx_client: mpsc::Sender<Result<Operation, Status>>,
	mut rx_core: broadcast::Receiver<Event>,
) {
	let mut queue: VecDeque<Operation> = VecDeque::new();
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
						// tx_client.send(Ok(op_net(&oop.1))).await.unwrap();
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
		let session_id: String;
		if let Some(sid) = req.metadata().get("session_id") {
			session_id = sid.to_str().unwrap().to_string();
		} else {
			return Err(Status::failed_precondition(
				"Missing metadata key 'session_id'",
			));
		}

		let path: String;
		if let Some(p) = req.metadata().get("path") {
			path = p.to_str().unwrap().to_string();
		} else {
			return Err(Status::failed_precondition("Missing metadata key 'path'"));
		}
		// TODO make these above nicer? more concise? idk

		if let Some(workspace) = self.state.workspaces_ref().get(&session_id) {
			let in_stream = req.into_inner();
			let (tx_og, rx) = mpsc::channel::<Result<Operation, Status>>(128);

			let b: BufferView = workspace.buffers.borrow().get(&path).unwrap().clone();
			let w = workspace.clone();
			tokio::spawn(async move {
				buffer_worker(b, in_stream, tx_og, w.bus.subscribe()).await;
			});

			// echo just write the same data that was received
			let out_stream = ReceiverStream::new(rx);

			return Ok(Response::new(Box::pin(out_stream) as Self::AttachStream));
		} else {
			return Err(Status::not_found(format!(
				"Norkspace with session_id {}",
				session_id
			)));
		}
	}

	async fn push(&self, _req:Request<BufferPayload>) -> Result<Response<BufferResponse>, Status> {
		todo!()
	}
	
	async fn pull(&self, _req:Request<BufferPayload>) -> Result<Response<BufferPayload>, Status> {
		todo!()
	}
	
}

impl BufferService {
	// TODO is this smart? Should I let main() instantiate servers?
	pub fn server(state: Arc<StateManager>) -> BufferServer<BufferService> {
		BufferServer::new(BufferService { state })
	}
}
