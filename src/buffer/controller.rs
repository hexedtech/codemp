use operational_transform::OperationSeq;
use tonic::{transport::Channel, Status, Streaming, async_trait};
use uuid::Uuid;

use crate::{
	ControllerWorker,
	cursor::tracker::{CursorTracker, CursorTrackerWorker},
	buffer::handle::{BufferHandle, OperationControllerEditor, OperationControllerWorker},
	proto::{buffer_client::BufferClient, BufferPayload, RawOp, OperationRequest},
};

#[derive(Clone)]
pub struct BufferController {
	id: String,
	client: BufferClient<Channel>,
}

impl From::<BufferClient<Channel>> for BufferController {
	fn from(value: BufferClient<Channel>) -> Self {
		BufferController { id: Uuid::new_v4().to_string(), client: value }
	}
}

impl BufferController {
	pub async fn new(dest: &str) -> Result<Self, tonic::transport::Error> {
		Ok(BufferClient::connect(dest.to_string()).await?.into())
	}

	pub fn id(&self) -> &str { &self.id }

	pub async fn create(&mut self, path: &str, content: Option<&str>) -> Result<bool, Status> {
		let req = BufferPayload {
			path: path.to_string(),
			content: content.map(|x| x.to_string()),
			user: self.id.clone(),
		};

		let res = self.client.create(req).await?;

		Ok(res.into_inner().accepted)
	}

	pub async fn listen(&mut self) -> Result<CursorTracker, Status> {
		let req = BufferPayload {
			path: "".into(),
			content: None,
			user: self.id.clone(),
		};

		let stream = self.client.listen(req).await?.into_inner();

		let controller = CursorTrackerWorker::new(self.id().to_string());
		let handle = controller.subscribe();
		let client = self.client.clone();

		tokio::spawn(async move {
			tracing::debug!("cursor worker started");
			controller.work(stream, client).await;
			tracing::debug!("cursor worker stopped");
		});

		Ok(handle)
	}

	pub async fn attach(&mut self, path: &str) -> Result<BufferHandle, Status> {
		let req = BufferPayload {
			path: path.to_string(),
			content: None,
			user: self.id.clone(),
		};

		let content = self.client.sync(req.clone())
			.await?
			.into_inner()
			.content
			.unwrap_or("".into());

		let stream = self.client.attach(req).await?.into_inner();

		let controller = OperationControllerWorker::new((self.clone(), stream), &content, path);
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
impl OperationControllerEditor for (BufferController, Streaming<RawOp>) {
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
