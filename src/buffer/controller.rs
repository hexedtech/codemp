use tonic::{transport::Channel, Status};
use uuid::Uuid;

use crate::{
	ControllerWorker,
	buffer::handle::{BufferHandle, OperationControllerWorker},
	proto::{buffer_client::BufferClient, BufferPayload},
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

	pub async fn create(&mut self, path: &str, content: Option<&str>) -> Result<(), Status> {
		let req = BufferPayload {
			path: path.to_string(),
			content: content.map(|x| x.to_string()),
			user: self.id.clone(),
		};

		self.client.create(req).await?;

		Ok(())
	}

	// pub async fn listen(&mut self) -> Result<CursorTracker, Status> {
	// 	let req = BufferPayload {
	// 		path: "".into(),
	// 		content: None,
	// 		user: self.id.clone(),
	// 	};

	// 	let stream = self.client.listen(req).await?.into_inner();

	// 	let controller = CursorTrackerWorker::new(self.id().to_string());
	// 	let handle = controller.subscribe();
	// 	let client = self.client.clone();

	// 	tokio::spawn(async move {
	// 		tracing::debug!("cursor worker started");
	// 		controller.work(stream, client).await;
	// 		tracing::debug!("cursor worker stopped");
	// 	});

	// 	Ok(handle)
	// }

	pub async fn attach(&mut self, path: &str) -> Result<BufferHandle, Status> {
		let req = BufferPayload {
			path: path.to_string(),
			content: None,
			user: self.id.clone(),
		};

		let content = self.client.sync(req.clone())
			.await?
			.into_inner()
			.content;

		let stream = self.client.attach(req).await?.into_inner();

		let controller = OperationControllerWorker::new(self.id().to_string(), &content, path);
		let factory = controller.subscribe();
		let client = self.client.clone();

		tokio::spawn(async move {
			tracing::debug!("buffer worker started");
			controller.work(client, stream).await;
			tracing::debug!("buffer worker stopped");
		});

		Ok(factory)
	}
}
