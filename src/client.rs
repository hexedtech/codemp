use operational_transform::OperationSeq;
use tonic::{transport::Channel, Status, Streaming, async_trait};
use uuid::Uuid;

use crate::{
	controller::{ControllerWorker,
		cursor::{CursorControllerHandle, CursorControllerWorker, CursorEditor},
		buffer::{OperationControllerHandle, OperationControllerEditor, OperationControllerWorker}
	},
	proto::{buffer_client::BufferClient, BufferPayload, RawOp, OperationRequest, Cursor},
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
	pub async fn new(dest: &str) -> Result<Self, tonic::transport::Error> {
		Ok(BufferClient::connect(dest.to_string()).await?.into())
	}

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

		let stream = self.client.listen(req).await?.into_inner();

		let controller = CursorControllerWorker::new(self.id().to_string(), (self.clone(), stream));
		let handle = controller.subscribe();

		tokio::spawn(async move {
			tracing::debug!("cursor worker started");
			controller.work().await;
			tracing::debug!("cursor worker stopped");
		});

		Ok(handle)
	}

	pub async fn attach(&mut self, path: String) -> Result<OperationControllerHandle, Status> {
		let req = BufferPayload {
			path: path.clone(),
			content: None,
			user: self.id.clone(),
		};

		let content = self.client.sync(req.clone())
			.await?
			.into_inner()
			.content
			.unwrap_or("".into());

		let stream = self.client.attach(req).await?.into_inner();

		let controller = OperationControllerWorker::new((self.clone(), stream), content, path);
		let factory = controller.subscribe();

		tokio::spawn(async move {
			tracing::debug!("buffer worker started");
			controller.work().await;
			tracing::debug!("buffer worker stopped");
		});

		Ok(factory)
	}
}

#[async_trait]
impl OperationControllerEditor for (CodempClient, Streaming<RawOp>) {
	async fn edit(&mut self, path: String, op: OperationSeq) -> bool {
		let req = OperationRequest {
			hash: "".into(),
			opseq: serde_json::to_string(&op).unwrap(),
			path,
			user: self.0.id().to_string(),
		};
		match self.0.client.edit(req).await {
			Ok(res) => res.into_inner().accepted,
			Err(e) => {
				tracing::error!("error sending edit: {}", e);
				false
			}
		}
	}

	async fn recv(&mut self) -> Option<OperationSeq> {
		match self.1.message().await {
			Ok(Some(op)) => Some(serde_json::from_str(&op.opseq).unwrap()),
			Ok(None) => None,
			Err(e) => {
				tracing::error!("could not receive edit from server: {}", e);
				None
			}
		}
	}
}

#[async_trait]
impl CursorEditor for (CodempClient, Streaming<Cursor>) {
	async fn moved(&mut self, cursor: Cursor) -> bool {
		match self.0.client.moved(cursor).await {
			Ok(res) => res.into_inner().accepted,
			Err(e) => {
				tracing::error!("could not send cursor movement: {}", e);
				false
			}
		}
	}

	async fn recv(&mut self) -> Option<Cursor> {
		match self.1.message().await {
			Ok(cursor) => cursor,
			Err(e) => {
				tracing::error!("could not receive cursor update: {}", e);
				None
			}
		}
	}
}
