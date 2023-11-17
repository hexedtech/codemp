//! ### client
//! 
//! codemp client manager, containing grpc services

use std::{sync::Arc, collections::BTreeMap};
use futures::stream::FuturesUnordered;

use tokio_stream::StreamExt;
use tonic::transport::Channel;

use crate::{
	api::Controller,
	cursor::{worker::CursorControllerWorker, controller::CursorController},
	proto::{
		buffer_client::BufferClient, cursor_client::CursorClient, UserIdentity, BufferPayload,
	},
	Error, api::controller::ControllerWorker,
	buffer::{controller::BufferController, worker::BufferControllerWorker},
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
	pub async fn join(&mut self, _session: &str) -> crate::Result<Arc<CursorController>> {
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
	pub async fn create(&mut self, path: &str, content: Option<&str>) -> crate::Result<()> {
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
	/// to interact with such buffer use [crate::api::Controller::send] or 
	/// [crate::api::Controller::recv] to exchange [crate::api::TextChange]
	pub async fn attach(&mut self, path: &str) -> crate::Result<Arc<BufferController>> {
		if let Some(workspace) = &mut self.workspace {
			let mut client = self.client.buffer.clone();
			let req = BufferPayload {
				path: path.to_string(), user: self.id.clone(), content: None
			};

			let stream = client.attach(req).await?.into_inner();

			let controller = BufferControllerWorker::new(self.id.clone(), path);
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


	pub async fn select_buffer(&self) -> crate::Result<String> {
		let mut futures = FuturesUnordered::new();
		match &self.workspace {
			None => Err(Error::InvalidState { msg: "join workspace first".into() }),
			Some(workspace) => {
				for (id, buffer) in workspace.buffers.iter() {
					futures.push(async move {
						buffer.poll().await?;
						Ok::<&String, Error>(id)
					})
				}
				match futures.next().await {
					None => Err(Error::Deadlocked), // TODO shouldn't really happen???
					Some(x) => Ok(x?.clone()),
				}
			}
		}
	}
}
