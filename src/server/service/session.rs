pub mod proto {
	tonic::include_proto!("session");
}

use std::sync::Arc;

use proto::{session_server::Session, WorkspaceBuilderRequest, SessionResponse};
use tonic::{Request, Response, Status};


use crate::actor::{
	state::StateManager, workspace::Workspace, // TODO fuck x2!
};

use self::proto::session_server::SessionServer;

#[derive(Debug)]
pub struct SessionService {
	state: Arc<StateManager>,
}

#[tonic::async_trait]
impl Session for SessionService {
	async fn create_workspace(
		&self,
		_req: Request<WorkspaceBuilderRequest>,
	) -> Result<Response<SessionResponse>, Status> {
		// let name = req.extensions().get::<WorkspaceExtension>().unwrap().id.clone();
		let w = Workspace::new("im lazy".into());
		let res = SessionResponse { accepted:true, session_key: w.id.to_string() };

		self.state.view().add(w).await;
		Ok(Response::new(res))
	}

	// async fn authenticate(
	// 	&self,
	// 	req: Request<SessionRequest>,
	// ) -> Result<Response<SessionResponse>, Status> {
	// 	todo!()
	// }

	// async fn list_workspaces(
	// 	&self,
	// 	req: Request<SessionRequest>,
	// ) -> Result<Response<WorkspaceList>, Status> {
	// 	todo!()
	// }
}

impl SessionService {
	pub fn new(state: Arc<StateManager>) -> SessionService {
		SessionService { state }
	}

	pub fn server(self) -> SessionServer<SessionService> {
		SessionServer::new(self)
	}
}
