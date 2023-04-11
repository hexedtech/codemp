/// TODO better name for this file

use std::sync::Arc;
use tracing::{error, warn};
use uuid::Uuid;

use crate::{
	opfactory::AsyncFactory,
	proto::{buffer_client::BufferClient, BufferPayload, OperationRequest, RawOp},
	tonic::{transport::Channel, Status, Streaming},
};

impl From::<BufferClient<Channel>> for CodempClient {
	fn from(x: BufferClient<Channel>) -> CodempClient {
		CodempClient {
			id: Uuid::new_v4(),
			client:x,
			factory: Arc::new(AsyncFactory::new(None)),
		}
	}
}

#[derive(Clone)]
pub struct CodempClient {
	id: Uuid,
	client: BufferClient<Channel>,
	factory: Arc<AsyncFactory>,
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
		match self.factory.insert(txt, pos).await {
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
		match self.factory.delete(pos, count).await {
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

	pub async fn sync(&mut self, path: String) -> Result<String, Status> {
		let res = self.client.sync(
			BufferPayload {
				path, content: None, user: self.id.to_string(),
			}
		).await?;
		Ok(res.into_inner().content.unwrap_or("".into()))
	}

	async fn worker<F : Fn(String) -> ()>(mut stream: Streaming<RawOp>, factory: Arc<AsyncFactory>, callback: F) {
		loop {
			match stream.message().await {
				Err(e) => break error!("error receiving change: {}", e),
				Ok(v) => match v {
					None => break warn!("stream closed"),
					Some(operation) => {
						match serde_json::from_str(&operation.opseq) {
							Err(e) => break error!("could not deserialize opseq: {}", e),
							Ok(op) => match factory.process(op).await {
								Err(e) => break error!("desynched: {}", e),
								Ok(x) => callback(x),
							},
						}
					}
				},
			}
		}
	}
}
