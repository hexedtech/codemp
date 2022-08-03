use std::{pin::Pin, sync::Arc};

// use tracing::{debug, error, info, warn};

use tonic::{Request, Response, Status};

pub mod proto {
	tonic::include_proto!("workspace");
}

use tokio_stream::Stream; // TODO example used this?

use proto::workspace_server::{Workspace, WorkspaceServer};
use proto::{BufferList, Event, WorkspaceRequest};

use crate::actor::state::StateManager;

type EventStream = Pin<Box<dyn Stream<Item = Result<Event, Status>> + Send>>;

#[derive(Debug)]
pub struct WorkspaceService {
	state: Arc<StateManager>,
}

#[tonic::async_trait]
impl Workspace for WorkspaceService {
	type SubscribeStream = EventStream;

	async fn subscribe(
		&self,
		_req: Request<WorkspaceRequest>,
	) -> Result<tonic::Response<EventStream>, Status> {
		todo!()
	}

	async fn buffers(
		&self,
		req: Request<WorkspaceRequest>,
	) -> Result<Response<BufferList>, Status> {
		let r = req.into_inner();
		match self.state.workspaces_ref().get(&r.session_key) {
			Some(w) => {
				let mut out = Vec::new();
				for (_k, v) in w.buffers_ref().iter() {
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
}

impl WorkspaceService {
	pub fn server(state: Arc<StateManager>) -> WorkspaceServer<WorkspaceService> {
		WorkspaceServer::new(WorkspaceService { state })
	}
}
