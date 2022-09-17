use std::{pin::Pin, sync::Arc};

use tonic::codegen::InterceptedService;
use tonic::service::Interceptor;
use tracing::debug;

use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use tokio::sync::{watch, mpsc};

pub mod proto {
	tonic::include_proto!("workspace");
}

use tokio_stream::Stream; // TODO example used this?

use proto::workspace_server::{Workspace, WorkspaceServer};
use proto::{BufferList, Event, WorkspaceRequest, WorkspaceResponse, UsersList, BufferRequest};

use crate::actor::{buffer::Buffer, state::StateManager, workspace::Workspace as WorkspaceInstance}; // TODO fuck x2!

struct WorkspaceExtension {
	id: String
}

#[derive(Debug, Clone)]
pub struct WorkspaceInterceptor {
	state: Arc<StateManager>,
}

impl Interceptor for WorkspaceInterceptor {
	fn call(&mut self, mut req: tonic::Request<()>) -> Result<tonic::Request<()>, Status> {
		// Set an extension that can be retrieved by `say_hello`
		let id;

		// TODO this is kinda spaghetti but I can't borrow immutably and mutably req inside this match
		// tree...
		match req.metadata().get("workspace") {
			Some(value) => {
				match value.to_str() {
					Ok(w_id) => {
						id = w_id.to_string();
					},
					Err(_) => return Err(Status::invalid_argument("Workspace key is not valid")),
				}
			},
			None => return Err(Status::unauthenticated("No workspace key included in request"))
		}

		if !self.state.workspaces.borrow().contains_key(&id) {
			return Err(Status::not_found(format!("Workspace '{}' could not be found", id)));
		}

		req.extensions_mut().insert(WorkspaceExtension { id });
		Ok(req)
	}
}




type EventStream = Pin<Box<dyn Stream<Item = Result<Event, Status>> + Send>>;

#[derive(Debug)]
pub struct WorkspaceService {
	state: Arc<StateManager>,
}

#[tonic::async_trait]
impl Workspace for WorkspaceService {
	type SubscribeStream = EventStream;

	async fn create(
		&self,
		request: Request<WorkspaceRequest>,
	) -> Result<Response<WorkspaceResponse>, Status> {
		debug!("create request: {:?}", request);
		// We should always have an extension because of the interceptor but maybe don't unwrap?
		let ext = request.extensions().get::<WorkspaceExtension>().unwrap();
		let r = request.into_inner();

		let _w = WorkspaceInstance::new(ext.id);

		let reply = WorkspaceResponse {
			// session_key: r.session_key.clone(),
			accepted: true,
		};

		// self.tx.send(AlterState::ADD{key: r.session_key.clone(), w}).await.unwrap();

		Ok(Response::new(reply))
	}

	async fn subscribe(
		&self,
		req: Request<WorkspaceRequest>,
	) -> Result<tonic::Response<EventStream>, Status> {
		let r = req.into_inner();
		match self.state.get(&r.session_key) {
			Some(w) => {
				let bus_clone = w.bus.clone();
				let (_stop_tx, stop_rx) = watch::channel(true);
				let (tx, rx) = mpsc::channel::<Result<Event, Status>>(128);
				tokio::spawn(async move {
					let mut event_receiver = bus_clone.subscribe();
					while stop_rx.borrow().to_owned() {
						let _res = event_receiver.recv().await.unwrap();
						let broadcasting = Event { id: 1, body: Some("".to_string()) }; // TODO actually process packet
						tx.send(Ok(broadcasting)).await.unwrap();
					}
				});
				return Ok(Response::new(Box::pin(ReceiverStream::new(rx))));
			},
			None => Err(Status::not_found(format!(
				"No active workspace with session_key '{}'",
				r.session_key
			)))
		}
	}

	async fn buffers(
		&self,
		req: Request<WorkspaceRequest>,
	) -> Result<Response<BufferList>, Status> {
		let r = req.into_inner();
		match self.state.get(&r.session_key) {
			Some(w) => {
				let mut out = Vec::new();
				for (_k, v) in w.buffers.borrow().iter() {
					out.push(v.name.clone());
				}
				Ok(Response::new(BufferList { path: out }))
			}
			None => Err(Status::not_found(format!(
				"No active workspace with session_key '{}'",
				r.session_key
			))),
		}
	}

	async fn new_buffer(
		&self,
		req: Request<BufferRequest>,
	) -> Result<Response<WorkspaceResponse>, Status> {
		let r = req.into_inner();
		if let Some(w) = self.state.get(&r.session_key) {
			let mut view = w.view();
			let buf = Buffer::new(r.path, w.bus.clone());
			view.buffers.add(buf).await;

			Ok(Response::new(WorkspaceResponse { accepted: true }))
		} else {
			return Err(Status::not_found(format!(
				"No active workspace with session_key '{}'",
				r.session_key
			)));
		}
	}

	async fn remove_buffer(
		&self,
		req: Request<BufferRequest>,
	) -> Result<Response<WorkspaceResponse>, Status> {
		let r = req.into_inner();
		match self.state.get(&r.session_key) {
			Some(w) => {
				let mut out = Vec::new();
				for (_k, v) in w.buffers.borrow().iter() {
					out.push(v.name.clone());
				}
				Ok(Response::new(WorkspaceResponse { accepted: true }))
			}
			None => Err(Status::not_found(format!(
				"No active workspace with session_key '{}'",
				r.session_key
			))),
		}
	}

	async fn list_users(
		&self,
		req: Request<WorkspaceRequest>,
	) -> Result<Response<UsersList>, Status> {
		let r = req.into_inner();
		match self.state.get(&r.session_key) {
			Some(w) => {
				let mut out = Vec::new();
				for (_k, v) in w.users.borrow().iter() {
					out.push(v.name.clone());
				}
				Ok(Response::new(UsersList { name: out }))
			},
			None => Err(Status::not_found(format!(
				"No active workspace with session_key '{}'",
				r.session_key
			))),
		}
	}

}

impl WorkspaceService {
	pub fn new(state: Arc<StateManager>) -> WorkspaceService {
		WorkspaceService { state }
	}

	pub fn server(self) -> InterceptedService<WorkspaceServer<WorkspaceService>, WorkspaceInterceptor> {
		let state = self.state.clone();
		WorkspaceServer::with_interceptor(self, WorkspaceInterceptor { state })
	}
}
