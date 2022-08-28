pub mod proto {
	tonic::include_proto!("session");
}

use std::sync::Arc;

use tracing::debug;

use tonic::{Request, Response, Status};

use proto::session_server::Session;
use proto::{SessionRequest, SessionResponse};

use crate::actor::{
	state::{AlterState, StateManager},
	workspace::Workspace as WorkspaceInstance, // TODO fuck x2!
};

#[derive(Debug)]
pub struct SessionService {
	state: Arc<StateManager>,
}

// #[tonic::async_trait]
// impl Session for SessionService {
// }
