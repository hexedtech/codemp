use std::{pin::Pin, sync::Arc};

use tonic::codegen::InterceptedService;
use tonic::service::Interceptor;
use tracing::debug;

use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status, Streaming};
use tokio::sync::{watch, mpsc};

pub mod proto {
	tonic::include_proto!("workspace");
}

use library::user::User;

use tokio_stream::{Stream, StreamExt}; // TODO example used this?

use proto::workspace_server::{Workspace, WorkspaceServer};
use proto::{BufferList, WorkspaceEvent, WorkspaceRequest, WorkspaceResponse, UsersList, BufferRequest, CursorUpdate, JoinRequest};

use library::user::UserCursor;
use crate::actor::{buffer::Buffer, state::StateManager}; // TODO fuck x2!

pub struct WorkspaceExtension {
	pub id: String
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


type EventStream = Pin<Box<dyn Stream<Item = Result<WorkspaceEvent, Status>> + Send>>;
type CursorUpdateStream = Pin<Box<dyn Stream<Item = Result<CursorUpdate, Status>> + Send>>;

#[derive(Debug)]
pub struct WorkspaceService {
	state: Arc<StateManager>,
}

#[tonic::async_trait]
impl Workspace for WorkspaceService {
	type JoinStream = EventStream;
	type SubscribeStream = CursorUpdateStream;

	async fn join(
		&self,
		req: Request<JoinRequest>,
	) -> Result<tonic::Response<Self::JoinStream>, Status> {
		let session_id = req.extensions().get::<WorkspaceExtension>().unwrap().id.clone();
		let r = req.into_inner();
		let run = self.state.run.clone();
		let user_name = r.name.clone();
		match self.state.get(&session_id) {
			Some(w) => {
				let (tx, rx) = mpsc::channel::<Result<WorkspaceEvent, Status>>(128);
				tokio::spawn(async move {
					let mut event_receiver = w.bus.subscribe();
					w.view().users.add(
						crate::actor::state::User {
							name: "some-name".to_string(), // get from request
							cursor: UserCursor { buffer:0, x:0, y:0 }
						}
					);
					while run.borrow().to_owned() {
						let res = event_receiver.recv().await.unwrap();
						let broadcasting = WorkspaceEvent { id: 1, body: Some(res.to_string()) }; // TODO actually process packet
						tx.send(Ok(broadcasting)).await.unwrap();
					}
					w.view().users.remove(user_name);
				});
				return Ok(Response::new(Box::pin(ReceiverStream::new(rx))));
			},
			None => Err(Status::not_found(format!(
				"No active workspace with session_key '{}'",
				session_id
			)))
		}
	}

	async fn subscribe(
		&self,
		req: tonic::Request<Streaming<CursorUpdate>>,
	) -> Result<Response<Self::SubscribeStream>, Status> {
		let s_id = req.extensions().get::<WorkspaceExtension>().unwrap().id.clone();
		let mut r = req.into_inner();
		match self.state.get(&s_id) {
			Some(w) => {
				let cursors_ref = w.cursors.clone();
				let (_stop_tx, stop_rx) = watch::channel(true);
				let (tx, rx) = mpsc::channel::<Result<CursorUpdate, Status>>(128);
				tokio::spawn(async move {
					let mut workspace_bus = cursors_ref.subscribe();
					while stop_rx.borrow().to_owned() {
						tokio::select!{
							remote = workspace_bus.recv() => {
								if let Ok(cur) = remote {
									tx.send(Ok(cur)).await.unwrap();
								}
							},
							local = r.next() => {
								match local {
									Some(request) => {
										match request {
											Ok(cur) => {
												cursors_ref.send(cur).unwrap();
											},
											Err(e) => {},
										}
									},
									None => {},
								}
							},
						}
					}
				});
				return Ok(Response::new(Box::pin(ReceiverStream::new(rx))));
			},
			None => Err(Status::not_found(format!(
				"No active workspace with session_key '{}'",
				s_id
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
		let session_id = req.extensions().get::<WorkspaceExtension>().unwrap().id.clone();
		let r = req.into_inner();
		if let Some(w) = self.state.get(&session_id) {
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
		let session_id = req.extensions().get::<WorkspaceExtension>().unwrap().id.clone();
		let r = req.into_inner();
		match self.state.get(&session_id) {
			Some(w) => {
				w.view().buffers.remove(r.path);
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
		let session_id = req.extensions().get::<WorkspaceExtension>().unwrap().id.clone();
		let r = req.into_inner();
		match self.state.get(&session_id) {
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
