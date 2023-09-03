//! ### client
//! 
//! codemp client manager, containing grpc services

use std::{sync::Arc, collections::BTreeMap};

use tonic::transport::Channel;

use crate::{
	cursor::{worker::CursorControllerWorker, controller::CursorController},
	proto::{
		buffer_client::BufferClient, cursor_client::CursorClient, UserIdentity, BufferPayload,
	},
	Error, api::ControllerWorker, buffer::{controller::BufferController, worker::BufferControllerWorker},
};


/// codemp client manager
///
/// contains all required grpc services and the unique user id
/// will disconnect when dropped
/// can be used to interact with server
pub struct Client {
	id: String,
	client: Services,
	workspace: Option<Workspace>,
}

struct Services {
	buffer: BufferClient<Channel>,
	cursor: CursorClient<Channel>,
}

struct Workspace {
	cursor: Arc<CursorController>,
	buffers: BTreeMap<String, Arc<BufferController>>,
}


impl Client {
	/// instantiate and connect a new client
	pub async fn new(dst: &str) -> Result<Self, tonic::transport::Error> {
		let buffer = BufferClient::connect(dst.to_string()).await?;
		let cursor = CursorClient::connect(dst.to_string()).await?;
		let id = uuid::Uuid::new_v4().to_string();
		
		Ok(Client { id, client: Services { buffer, cursor}, workspace: None })
	}

	/// return a reference to current cursor controller, if currently in a workspace
	pub fn get_cursor(&self) -> Option<Arc<CursorController>> {
		Some(self.workspace.as_ref()?.cursor.clone())
	}

	/// leave current workspace if in one, disconnecting buffer and cursor controllers
	pub fn leave_workspace(&mut self) {
		// TODO need to stop tasks?
		self.workspace = None
	}

	/// disconnect from a specific buffer
	pub fn disconnect_buffer(&mut self, path: &str) -> bool {
		match &mut self.workspace {
			Some(w) => w.buffers.remove(path).is_some(),
			None => false,
		}
	}

	/// get a new reference to a buffer controller, if any is active to given path
	pub fn get_buffer(&self, path: &str) -> Option<Arc<BufferController>> {
		self.workspace.as_ref()?.buffers.get(path).cloned()
	}

	/// join a workspace, starting a cursorcontroller and returning a new reference to it
	/// 
	/// to interact with such workspace [crate::api::Controller::send] cursor events or
	/// [crate::api::Controller::recv] for events on the associated [crate::cursor::Controller].
	pub async fn join(&mut self, _session: &str) -> Result<Arc<CursorController>, Error> {
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

	/// create a new buffer in current workspace, with optional given content
	pub async fn create(&mut self, path: &str, content: Option<&str>) -> Result<(), Error> {
		if let Some(_workspace) = &self.workspace {
			self.client.buffer
				.create(BufferPayload {
					user: self.id.clone(),
					path: path.to_string(),
					content: content.map(|x| x.to_string()),
				}).await?;

			Ok(())
		} else {
			Err(Error::InvalidState { msg: "join a workspace first".into() })
		}
	}

	/// attach to a buffer, starting a buffer controller and returning a new reference to it
	/// 
	/// to interact with such buffer [crate::api::Controller::send] operation sequences 
	/// or [crate::api::Controller::recv] for text events using its [crate::buffer::Controller].
	/// to generate operation sequences use the [crate::buffer::OperationFactory]
	/// methods, which are implemented on [crate::buffer::Controller], such as
	/// [crate::buffer::OperationFactory::delta].
	pub async fn attach(&mut self, path: &str) -> Result<Arc<BufferController>, Error> {
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
			Err(Error::InvalidState { msg: "join a workspace first".into() })
		}
	}
}
