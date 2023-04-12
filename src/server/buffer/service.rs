use std::{pin::Pin, sync::{Arc, RwLock}, collections::HashMap};

use tokio::sync::{mpsc, broadcast};
use tonic::{Request, Response, Status};

use tokio_stream::{Stream, wrappers::ReceiverStream}; // TODO example used this?

use codemp::proto::{buffer_server::{Buffer, BufferServer}, RawOp, BufferPayload, BufferResponse, OperationRequest, CursorMov};
use tracing::info;

use super::actor::{BufferHandle, BufferStore};

type OperationStream = Pin<Box<dyn Stream<Item = Result<RawOp, Status>> + Send>>;
type CursorStream    = Pin<Box<dyn Stream<Item = Result<CursorMov, Status>> + Send>>;

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
	cursor: broadcast::Sender<CursorMov>,
}

impl BufferService {
	#[allow(unused)]
	fn get_buffer(&self, path: &String) -> Result<BufferHandle, Status> {
		match self.map.read().unwrap().get(path) {
			Some(buf) => Ok(buf.clone()),
			None => Err(Status::not_found("no buffer for given path")),
		}
	}
}

#[tonic::async_trait]
impl Buffer for BufferService {
	type AttachStream = OperationStream;
	type ListenStream = CursorStream;

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

	async fn listen(&self, req: Request<BufferPayload>) -> Result<tonic::Response<CursorStream>, Status> {
		let mut sub = self.cursor.subscribe();
		let myself = req.into_inner().user;
		let (tx, rx) = mpsc::channel(128);
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
		info!("registered new subscriber to cursor updates");
		Ok(Response::new(Box::pin(output_stream)))
	}

	async fn cursor(&self, req:Request<CursorMov>) -> Result<Response<BufferResponse>, Status> {
		match self.cursor.send(req.into_inner()) {
			Ok(_) => Ok(Response::new(BufferResponse { accepted: true, content: None})),
			Err(e) => Err(Status::internal(format!("could not broadcast cursor update: {}", e))),
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
		match tx.send(request).await {
			Ok(()) => Ok(Response::new(BufferResponse { accepted: true, content: None })),
			Err(e) => Err(Status::internal(format!("error sending edit to buffer actor: {}", e))),
		}
	}
	
	async fn create(&self, req:Request<BufferPayload>) -> Result<Response<BufferResponse>, Status> {
		let request = req.into_inner();
		let _handle = self.map.write().unwrap().handle(request.path, request.content);
		info!("created new buffer");
		let answ = BufferResponse { accepted: true, content: None };
		Ok(Response::new(answ))
	}

	async fn sync(&self, req: Request<BufferPayload>) -> Result<Response<BufferResponse>, Status> {
		let request = req.into_inner();
		match self.map.read().unwrap().get(&request.path) {
			None => Err(Status::not_found("requested buffer does not exist")),
			Some(buf) => {
				info!("synching buffer");
				let answ = BufferResponse { accepted: true, content: Some(buf.content.borrow().clone()) };
				Ok(Response::new(answ))
			}
		}
	}
}

impl BufferService {
	pub fn new() -> BufferService {
		let (cur_tx, _cur_rx) = broadcast::channel(64); // TODO hardcoded capacity
		BufferService {
			map: Arc::new(RwLock::new(HashMap::new().into())),
			cursor: cur_tx,
		}
	}

	pub fn server(self) -> BufferServer<BufferService> {
		BufferServer::new(self)
	}
}
