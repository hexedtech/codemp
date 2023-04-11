/// TODO better name for this file

use std::{sync::{Arc, RwLock}, collections::BTreeMap};
use tracing::{error, warn, info};
use uuid::Uuid;

use crate::{
	opfactory::AsyncFactory,
	proto::{buffer_client::BufferClient, BufferPayload, OperationRequest, RawOp},
	tonic::{transport::Channel, Status, Streaming},
};

pub type FactoryStore = Arc<RwLock<BTreeMap<String, Arc<AsyncFactory>>>>;

impl From::<BufferClient<Channel>> for CodempClient {
	fn from(x: BufferClient<Channel>) -> CodempClient {
		CodempClient {
			id: Uuid::new_v4(),
			client:x,
			factories: Arc::new(RwLock::new(BTreeMap::new())),
		}
	}
}

#[derive(Clone)]
pub struct CodempClient {
	id: Uuid,
	client: BufferClient<Channel>,
	factories: FactoryStore,
}

impl CodempClient {
	fn get_factory(&self, path: &String) -> Result<Arc<AsyncFactory>, Status> {
		match self.factories.read().unwrap().get(path) {
			Some(f) => Ok(f.clone()),
			None => Err(Status::not_found("no active buffer for given path")),
		}
	}

	pub fn add_factory(&self, path: String, factory:Arc<AsyncFactory>) {
		self.factories.write().unwrap().insert(path, factory);
	}

	pub async fn create(&mut self, path: String, content: Option<String>) -> Result<bool, Status> {
		let req = BufferPayload {
			path: path.clone(),
			content: content.clone(),
			user: self.id.to_string(),
		};

		let res = self.client.create(req).await?.into_inner();

		Ok(res.accepted)
	}

	pub async fn insert(&mut self, path: String, txt: String, pos: u64) -> Result<bool, Status> {
		let factory = self.get_factory(&path)?;
		match factory.insert(txt, pos).await {
			Err(e) => Err(Status::internal(format!("invalid operation: {}", e))),
			Ok(op) => {
				let req = OperationRequest {
					path,
					hash: "".into(),
					user: self.id.to_string(),
					opseq: serde_json::to_string(&op)
						.map_err(|_| Status::invalid_argument("could not serialize opseq"))?,
				};
				let res = self.client.edit(req).await?.into_inner();
				Ok(res.accepted)
			},
		}
	}

	pub async fn delete(&mut self, path: String, pos: u64, count: u64) -> Result<bool, Status> {
		let factory = self.get_factory(&path)?;
		match factory.delete(pos, count).await {
			Err(e) => Err(Status::internal(format!("invalid operation: {}", e))),
			Ok(op) => {
				let req = OperationRequest {
					path,
					hash: "".into(),
					user: self.id.to_string(),
					opseq: serde_json::to_string(&op)
						.map_err(|_| Status::invalid_argument("could not serialize opseq"))?,
				};
				let res = self.client.edit(req).await?.into_inner();
				Ok(res.accepted)
			},
		}
	}

	pub async fn attach<F>(&mut self, path: String, callback: F) -> Result<String, Status>
	where F : Fn(String) -> () + Send + 'static {
		let content = self.sync(path.clone()).await?;
		let factory = Arc::new(AsyncFactory::new(Some(content.clone())));
		self.add_factory(path.clone(), factory.clone());
		let req = BufferPayload {
			path,
			content: None,
			user: self.id.to_string(),
		};
		let stream = self.client.attach(req).await?.into_inner();
		tokio::spawn(async move { Self::worker(stream, factory, callback).await } );
		Ok(content)
	}

	pub fn detach(&mut self, path: String) {
		self.factories.write().unwrap().remove(&path);
	}

	async fn sync(&mut self, path: String) -> Result<String, Status> {
		let res = self.client.sync(
			BufferPayload {
				path, content: None, user: self.id.to_string(),
			}
		).await?;
		Ok(res.into_inner().content.unwrap_or("".into()))
	}

	async fn worker<F>(mut stream: Streaming<RawOp>, factory: Arc<AsyncFactory>, callback: F)
	where F : Fn(String) -> () {
		info!("|> buffer worker started");
		loop {
			match stream.message().await {
				Err(e) => break error!("error receiving change: {}", e),
				Ok(v) => match v {
					None => break warn!("stream closed"),
					Some(operation) => match serde_json::from_str(&operation.opseq) {
						Err(e) => break error!("could not deserialize opseq: {}", e),
						Ok(op) => match factory.process(op).await {
							Err(e) => break error!("desynched: {}", e),
							Ok(x) => callback(x),
						},
					}
				},
			}
		}
		info!("[] buffer worker stopped");
	}
}
