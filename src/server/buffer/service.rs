use std::{pin::Pin, sync::{Arc, RwLock}, collections::HashMap};

use tokio::sync::mpsc;
use tonic::{Request, Response, Status};

use tokio_stream::{Stream, wrappers::ReceiverStream}; // TODO example used this?

use codemp::proto::{buffer_server::{Buffer, BufferServer}, RawOp, BufferPayload, BufferResponse, OperationRequest};
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

#[tonic::async_trait]
impl Buffer for BufferService {
	type AttachStream = OperationStream;

	async fn attach(&self, req: Request<BufferPayload>) -> Result<tonic::Response<OperationStream>, Status> {
		let request = req.into_inner();
		let myself = request.user;
		match self.map.read().unwrap().get(&request.path) {
			Some(handle) => {
				let (tx, rx) = mpsc::channel(128);
				let mut sub = handle.subscribe();
				tokio::spawn(async move {
					loop {
						match sub.recv().await {
							Ok(v) => {
								if v.user == myself { continue }
								tx.send(Ok(v)).await.unwrap(); // TODO unnecessary channel?
							}
							Err(_e) => break,
						}
					}
				});
				let output_stream = ReceiverStream::new(rx);
				info!("registered new subscriber on buffer");
				Ok(Response::new(Box::pin(output_stream)))
			},
			None => Err(Status::not_found("path not found")),
		}
	}

	async fn edit(&self, req:Request<OperationRequest>) -> Result<Response<BufferResponse>, Status> {
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
		tx.send(request).await.unwrap();
		Ok(Response::new(BufferResponse { accepted: true }))
	}
	
	async fn create(&self, req:Request<BufferPayload>) -> Result<Response<BufferResponse>, Status> {
		let request = req.into_inner();
		let _handle = self.map.write().unwrap().handle(request.path, request.content);
		info!("created new buffer");
		let answ = BufferResponse { accepted: true };
		Ok(Response::new(answ))
	}
	
}

impl BufferService {
	pub fn new() -> BufferService {
		BufferService {
			map: Arc::new(RwLock::new(HashMap::new().into())),
		}
	}

	pub fn server(self) -> BufferServer<BufferService> {
		BufferServer::new(self)
	}
}
