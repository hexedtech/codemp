/// TODO better name for this file

use std::sync::{Arc, Mutex};
use tracing::{error, warn};
use uuid::Uuid;

use crate::{
	opfactory::OperationFactory,
	proto::{buffer_client::BufferClient, BufferPayload, OperationRequest, RawOp},
	tonic::{transport::Channel, Status, Streaming},
};

type FactoryHandle = Arc<Mutex<OperationFactory>>;

impl From::<BufferClient<Channel>> for CodempClient {
	fn from(x: BufferClient<Channel>) -> CodempClient {
		CodempClient {
			id: Uuid::new_v4(),
			client:x,
			factory: Arc::new(Mutex::new(OperationFactory::new(None)))
		}
	}
}

#[derive(Clone)]
pub struct CodempClient {
	id: Uuid,
	client: BufferClient<Channel>,
	factory: FactoryHandle, // TODO less jank solution than Arc<Mutex>
}

impl CodempClient {
	pub async fn create(&mut self, path: String, content: Option<String>) -> Result<bool, Status> {
		Ok(
			self.client.create(
				BufferPayload {
					path,
					content,
					user: self.id.to_string(),
				}
			)
				.await?
				.into_inner()
				.accepted
		)
	}

	pub async fn insert(&mut self, path: String, txt: String, pos: u64) -> Result<bool, Status> {
		let res = { self.factory.lock().unwrap().insert(&txt, pos) };
		match res {
			Ok(op) => {
				Ok(
					self.client.edit(
						OperationRequest {
							path,
							hash: "".into(),
							opseq: serde_json::to_string(&op).unwrap(),
							user: self.id.to_string(),
						}
					)
						.await?
						.into_inner()
						.accepted
				)
			},
			Err(e) => Err(Status::internal(format!("invalid operation: {}", e))),
		}
	}

	pub async fn delete(&mut self, path: String, pos: u64, count: u64) -> Result<bool, Status> {
		let res = { self.factory.lock().unwrap().delete(pos, count) };
		match res {
			Ok(op) => {
				Ok(
					self.client.edit(
						OperationRequest {
							path,
							hash: "".into(),
							opseq: serde_json::to_string(&op).unwrap(),
							user: self.id.to_string(),
						}
					)
						.await?
						.into_inner()
						.accepted
				)
			},
			Err(e) => Err(Status::internal(format!("invalid operation: {}", e))),
		}
	}

	pub async fn attach<F : Fn(String) -> () + Send + 'static>(&mut self, path: String, callback: F) -> Result<(), Status> {
		let stream = self.client.attach(
				BufferPayload {
					path,
					content: None,
					user: self.id.to_string(),
				}
			)
				.await?
				.into_inner();

		let factory = self.factory.clone();
		tokio::spawn(async move { Self::worker(stream, factory, callback).await } );

		Ok(())
	}

	async fn worker<F : Fn(String) -> ()>(mut stream: Streaming<RawOp>, factory: FactoryHandle, callback: F) {
		loop {
			match stream.message().await {
				Ok(v) => match v {
					Some(operation) => {
						let op = serde_json::from_str(&operation.opseq).unwrap();
						let res = { factory.lock().unwrap().process(op) };
						match res {
							Ok(x) => callback(x),
							Err(e) => break error!("desynched: {}", e),
						}
					}
					None => break warn!("stream closed"),
				},
				Err(e) => break error!("error receiving change: {}", e),
			}
		}
	}

	pub fn content(&self) -> String {
		let factory = self.factory.lock().unwrap();
		factory.content()
	}
}
