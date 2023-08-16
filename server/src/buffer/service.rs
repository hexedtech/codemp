use std::{pin::Pin, sync::{Arc, RwLock}, collections::HashMap};

use tokio::sync::{mpsc, oneshot};
use tonic::{Request, Response, Status};

use tokio_stream::{Stream, wrappers::ReceiverStream}; // TODO example used this?

use codemp::proto::{buffer_server::Buffer, RawOp, BufferPayload, BufferResponse, OperationRequest, BufferEditResponse, BufferCreateResponse};
use tracing::info;

use super::actor::{BufferHandle, BufferStore};

type OperationStream = Pin<Box<dyn Stream<Item = Result<RawOp, Status>> + Send>>;

struct BufferMap {
	store: HashMap<String, BufferHandle>,
}

impl From::<HashMap<String, BufferHandle>> for BufferMap {
	fn from(value: HashMap<String, BufferHandle>) -> Self {
		BufferMap { store: value }
	}
}

impl BufferStore<String> for BufferMap {
	fn get(&self, key: &String) -> Option<&BufferHandle> {
		self.store.get(key)
	}
	fn put(&mut self, key: String, handle: BufferHandle) -> Option<BufferHandle> {
		self.store.insert(key, handle)
	}
}

pub struct BufferService {
	map: Arc<RwLock<BufferMap>>,
}

impl Default for BufferService {
	fn default() -> BufferService {
		BufferService {
			map: Arc::new(RwLock::new(HashMap::new().into())),
		}
	}
}

#[tonic::async_trait]
impl Buffer for BufferService {
	type AttachStream = OperationStream;

	async fn attach(&self, req: Request<BufferPayload>) -> Result<Response<OperationStream>, Status> {
		let request = req.into_inner();
		let myself = request.user;
		match self.map.read().unwrap().get(&request.path) {
			None => Err(Status::not_found("path not found")),
			Some(handle) => {
				let (tx, rx) = mpsc::channel(128);
				let mut sub = handle.subscribe();
				tokio::spawn(async move {
					while let Ok(v) = sub.recv().await {
						if v.user == myself { continue }
						tx.send(Ok(v)).await.unwrap(); // TODO unnecessary channel?
					}
				});
				let output_stream = ReceiverStream::new(rx);
				info!("registered new subscriber on buffer");
				Ok(Response::new(Box::pin(output_stream)))
			},
		}
	}

	async fn edit(&self, req:Request<OperationRequest>) -> Result<Response<BufferEditResponse>, Status> {
		let request = req.into_inner();
		let tx = match self.map.read().unwrap().get(&request.path) {
			Some(handle) => {
				// if format!("{:x}", *handle.digest.borrow()) != request.hash {
				// 	return Ok(Response::new(BufferResponse { accepted : false } ));
				// }
				handle.edit.clone()
			},
			None => return Err(Status::not_found("path not found")),
		};
		info!("sending edit to buffer: {}", request.opseq);
		let (ack, status) = oneshot::channel();
		match tx.send((ack, request)).await {
			Err(e) => Err(Status::internal(format!("error sending edit to buffer actor: {}", e))),
			Ok(()) => {
				match status.await {
					Ok(_accepted) => Ok(Response::new(BufferEditResponse { })),
					Err(e) => Err(Status::internal(format!("error receiving edit result: {}", e))),
				}
			}
		}
	}
	
	async fn create(&self, req:Request<BufferPayload>) -> Result<Response<BufferCreateResponse>, Status> {
		let request = req.into_inner();
		let _handle = self.map.write().unwrap().handle(request.path, request.content);
		info!("created new buffer");
		Ok(Response::new(BufferCreateResponse { }))
	}

	async fn sync(&self, req: Request<BufferPayload>) -> Result<Response<BufferResponse>, Status> {
		let request = req.into_inner();
		match self.map.read().unwrap().get(&request.path) {
			None => Err(Status::not_found("requested buffer does not exist")),
			Some(buf) => {
				info!("synching buffer");
				let answ = BufferResponse { content: buf.content.borrow().clone() };
				Ok(Response::new(answ))
			}
		}
	}
}
