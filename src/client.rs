use std::{sync::Arc, collections::BTreeMap};

use tonic::transport::Channel;

use crate::{
	cursor::{worker::CursorControllerWorker, controller::CursorController},
	proto::{
		buffer_client::BufferClient, cursor_client::CursorClient, UserIdentity, BufferPayload,
	},
	CodempError, ControllerWorker, buffer::{controller::BufferController, worker::BufferControllerWorker},
};


pub struct CodempClient {
	id: String,
	client: ServiceClients,
	workspace: Option<Workspace>,
}

struct ServiceClients {
	buffer: BufferClient<Channel>,
	cursor: CursorClient<Channel>,
}

struct Workspace {
	cursor: Arc<CursorController>,
	buffers: BTreeMap<String, Arc<BufferController>>,
}


impl CodempClient {
	pub async fn new(dst: &str) -> Result<Self, tonic::transport::Error> {
		let buffer = BufferClient::connect(dst.to_string()).await?;
		let cursor = CursorClient::connect(dst.to_string()).await?;
		let id = uuid::Uuid::new_v4().to_string();
		
		Ok(CodempClient { id, client: ServiceClients { buffer, cursor}, workspace: None })
	}

	pub fn get_cursor(&self) -> Option<Arc<CursorController>> {
		Some(self.workspace?.cursor.clone())
	}

	pub fn get_buffer(&self, path: &str) -> Option<Arc<BufferController>> {
		self.workspace?.buffers.get(path).cloned()
	}

	pub async fn join(&mut self, _session: &str) -> Result<Arc<CursorController>, CodempError> {
		// TODO there is no real workspace handling in codemp server so it behaves like one big global
		//  session. I'm still creating this to start laying out the proper use flow
		let stream = self.client.cursor.listen(UserIdentity { id: "".into() }).await?.into_inner();

		let controller = CursorControllerWorker::new(self.id.clone());
		let client = self.client.cursor.clone();

		let handle = Arc::new(controller.subscribe());

		tokio::spawn(async move {
			tracing::debug!("cursor worker started");
			controller.work(client, stream).await;
			tracing::debug!("cursor worker stopped");
		});

		self.workspace = Some(
			Workspace {
				cursor: handle.clone(),
				buffers: BTreeMap::new()
			}
		);

		Ok(handle)
	}

	pub async fn create(&mut self, path: &str, content: Option<&str>) -> Result<(), CodempError> {
		if let Some(workspace) = &self.workspace {
			self.client.buffer
				.create(BufferPayload {
					user: self.id.clone(),
					path: path.to_string(),
					content: content.map(|x| x.to_string()),
				}).await?;

			Ok(())
		} else {
			Err(CodempError::InvalidState { msg: "join a workspace first".into() })
		}
	}

	pub async fn attach(&mut self, path: &str, content: Option<&str>) -> Result<Arc<BufferController>, CodempError> {
		if let Some(workspace) = &mut self.workspace {
			let mut client = self.client.buffer.clone();
			let req = BufferPayload {
				path: path.to_string(), user: self.id.clone(), content: None
			};

			let content = client.sync(req.clone()).await?.into_inner().content;

			let stream = client.attach(req).await?.into_inner();

			let controller = BufferControllerWorker::new(self.id.clone(), &content, path);
			let handler = Arc::new(controller.subscribe());

			let _path = path.to_string();
			tokio::spawn(async move {
				tracing::debug!("buffer[{}] worker started", _path);
				controller.work(client, stream).await;
				tracing::debug!("buffer[{}] worker stopped", _path);
			});

			workspace.buffers.insert(path.to_string(), handler.clone());

			Ok(handler)
		} else {
			Err(CodempError::InvalidState { msg: "join a workspace first".into() })
		}
	}
}
