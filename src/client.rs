use std::sync::Arc;

use operational_transform::OperationSeq;
use tonic::{transport::Channel, Status};
use tracing::{error, warn, debug};
use uuid::Uuid;

use crate::{
	cursor::{CursorControllerHandle, CursorControllerWorker, CursorProvider},
	operation::{OperationProcessor, OperationController},
	proto::{buffer_client::BufferClient, BufferPayload, OperationRequest, CursorMov}, errors::IgnorableError,
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
	pub fn id(&self) -> &str { &self.id	}

	pub async fn create(&mut self, path: String, content: Option<String>) -> Result<bool, Status> {
		let req = BufferPayload {
			path, content,
			user: self.id.clone(),
		};

		let res = self.client.create(req).await?;

		Ok(res.into_inner().accepted)
	}

	pub async fn listen(&mut self) -> Result<CursorControllerHandle, Status> {
		let req = BufferPayload {
			path: "".into(),
			content: None,
			user: self.id.clone(),
		};

		let mut stream = self.client.listen(req).await?.into_inner();

		let mut controller = CursorControllerWorker::new(self.id().to_string());
		let handle = controller.subscribe();
		let mut _client = self.client.clone();

		tokio::spawn(async move {
			loop {
				tokio::select!{
					res = stream.message() => {
						match res {
							Err(e)      => break error!("error receiving cursor: {}", e),
							Ok(None)    => break debug!("cursor worker clean exit"),
							Ok(Some(x)) => { controller.broadcast(x); },
						}
					},
					Some(op) = controller.wait() => {
						_client.cursor(CursorMov::from(op)).await
							.unwrap_or_warn("could not send cursor update")
					}

				}
			}
		});

		Ok(handle)
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
		let _path = path.clone();

		tokio::spawn(async move {
			loop {
				if !_factory.run() { break debug!("downstream worker clean exit") }
				match stream.message().await {
					Err(e)      => break error!("error receiving update: {}", e),
					Ok(None)    => break warn!("stream closed for buffer {}", _path),
					Ok(Some(x)) => match serde_json::from_str::<OperationSeq>(&x.opseq) {
						Err(e)    => error!("error deserializing opseq: {}", e),
						Ok(v)     => match _factory.process(v) {
							Err(e)  => break error!("could not apply operation from server: {}", e),
							Ok(_range) => { } // range is obtained awaiting wait(), need to pass the OpSeq itself
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
				if !_factory.run() { break }
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
			debug!("upstream worker clean exit");
		});

		Ok(factory)
	}
}
