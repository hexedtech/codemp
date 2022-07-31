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
		req: Request<WorkspaceRequest>,
	) -> Result<tonic::Response<EventStream>, Status> {
		todo!()
	}

	async fn buffers(
		&self,
		req: Request<WorkspaceRequest>,
	) -> Result<Response<BufferList>, Status> {
		todo!()
	}
}

impl WorkspaceService {
	pub fn server(state: Arc<StateManager>) -> WorkspaceServer<WorkspaceService> {
		WorkspaceServer::new(WorkspaceService { state })
	}
}
