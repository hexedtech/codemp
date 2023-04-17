use std::sync::Arc;

use operational_transform::OperationSeq;
use tonic::{transport::Channel, Status};
use tracing::{error, warn};
use uuid::Uuid;

use crate::{
	cursor::{CursorController, CursorStorage},
	operation::{OperationController, OperationProcessor},
	proto::{buffer_client::BufferClient, BufferPayload, OperationRequest},
};

#[derive(Clone)]
pub struct CodempClient {
	id: String,
	client: BufferClient<Channel>,
}

impl From::<BufferClient<Channel>> for CodempClient {
	fn from(value: BufferClient<Channel>) -> Self {
		CodempClient { id: Uuid::new_v4().to_string(), client: value }
	}
}

impl CodempClient {
	pub fn new(id: String, client: BufferClient<Channel>) -> Self {
		CodempClient { id, client }

	}

	pub async fn create(&mut self, path: String, content: Option<String>) -> Result<bool, Status> {
		let req = BufferPayload {
			path, content,
			user: self.id.clone(),
		};

		let res = self.client.create(req).await?;

		Ok(res.into_inner().accepted)
	}

	pub async fn listen(&mut self) -> Result<Arc<CursorController>, Status> {
		let req = BufferPayload {
			path: "".into(),
			content: None,
			user: self.id.clone(),
		};

		let mut stream = self.client.listen(req).await?.into_inner();

		let controller = Arc::new(CursorController::new());

		let _controller = controller.clone();
		tokio::spawn(async move {
			loop {
				match stream.message().await {
					Err(e)      => break error!("error receiving cursor: {}", e),
					Ok(None)    => break,
					Ok(Some(x)) => { _controller.update(x); },
				}
			}
		});

		Ok(controller)
	}

	pub async fn attach(&mut self, path: String) -> Result<Arc<OperationController>, Status> {
		let req = BufferPayload {
			path: path.clone(),
			content: None,
			user: self.id.clone(),
		};

		let content = self.client.sync(req.clone())
			.await?
			.into_inner()
			.content;

		let mut stream = self.client.attach(req).await?.into_inner();

		let factory = Arc::new(OperationController::new(content.unwrap_or("".into())));

		let _factory = factory.clone();
		tokio::spawn(async move {
			loop {
				match stream.message().await {
					Err(e) => break error!("error receiving update: {}", e),
					Ok(None) => break, // clean exit
					Ok(Some(x)) => match serde_json::from_str::<OperationSeq>(&x.opseq) {
						Err(e) => break error!("error deserializing opseq: {}", e),
						Ok(v) => match _factory.process(v).await {
							Err(e) => break error!("could not apply operation from server: {}", e),
							Ok(_txt) => { }
						}
					},
				}
			}
		});

		let mut _client = self.client.clone();
		let _uid = self.id.clone();
		let _factory = factory.clone();
		let _path = path.clone();
		tokio::spawn(async move {
			while let Some(op) = _factory.poll().await {
				let req = OperationRequest {
					hash: "".into(),
					opseq: serde_json::to_string(&op).unwrap(),
					path: _path.clone(),
					user: _uid.clone(),
				};
				match _client.edit(req).await {
					Ok(res) => match res.into_inner().accepted {
						true => { _factory.ack().await; },
						false => {
							warn!("server rejected operation, retrying in 1s");
							tokio::time::sleep(std::time::Duration::from_secs(1)).await;
						}
					},
					Err(e) => error!("could not send edit: {}", e),
				}
			}
		});

		Ok(factory)
	}
}
