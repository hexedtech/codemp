use std::{collections::HashMap, sync::Arc};

use state::AlterState;
use tonic::{transport::Server, Request, Response, Status};

pub mod proto {
	tonic::include_proto!("workspace");
}

use tokio::sync::{mpsc, watch};

use proto::workspace_server::{Workspace, WorkspaceServer};
use proto::{SessionRequest, SessionResponse};

use crate::workspace::Workspace as WorkspaceInstance; // TODO fuck!

pub mod workspace;
pub mod state;

#[derive(Debug)]
pub struct WorkspaceService {
	tx: mpsc::Sender<AlterState>,
	rx: watch::Receiver<HashMap<String, Arc<WorkspaceInstance>>>
}

#[tonic::async_trait]
impl Workspace for WorkspaceService {
	async fn create(
		&self,
		request: Request<SessionRequest>,
	) -> Result<Response<SessionResponse>, Status> {
		println!("Got a request: {:?}", request);
		let r = request.into_inner();

		let w = WorkspaceInstance::new(r.session_key.clone(), r.content.unwrap_or("".to_string()));

		let reply = proto::SessionResponse {
			session_key: r.session_key.clone(),
			accepted: true,
			content: Some(w.content.clone()),
			hash: None,
		};

		self.tx.send(AlterState::ADD{key: r.session_key.clone(), w}).await.unwrap();

		Ok(Response::new(reply))
	}

	async fn sync(
		&self,
		request: Request<SessionRequest>,
	) -> Result<Response<SessionResponse>, Status> {
		println!("Got a request: {:?}", request);
		let r = request.into_inner();

		if let Some(w) = self.rx.borrow().get(&r.session_key) {
			let reply = proto::SessionResponse {
				session_key: r.session_key,
				accepted: true,
				content: Some(w.content.clone()),
				hash: None,
			};

			Ok(Response::new(reply))
		} else {
			Err(Status::out_of_range("fuck you".to_string()))
		}
	}

	// TODO make it do something
	async fn join(
		&self,
		request: Request<SessionRequest>,
	) -> Result<Response<SessionResponse>, Status> {
		println!("Got a request: {:?}", request);

		let reply = proto::SessionResponse {
			session_key: request.into_inner().session_key,
			accepted: true,
			content: None,
			hash: None,
		};

		Ok(Response::new(reply))
	}

	async fn leave(
		&self,
		request: Request<SessionRequest>,
	) -> Result<Response<SessionResponse>, Status> {
		println!("Got a request: {:?}", request);
		let r = request.into_inner();
		let mut removed = false;

		if self.rx.borrow().get(&r.session_key).is_some() {
			self.tx.send(AlterState::REMOVE { key: r.session_key.clone() }).await.unwrap();
			removed = true; // TODO this is a lie! Verify it
		}

		let reply = proto::SessionResponse {
			session_key: r.session_key,
			accepted: removed,
			content: None,
			hash: None,
		};

		Ok(Response::new(reply))
	}
	
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let addr = "[::1]:50051".parse()?;

	let (tx, rx) = state::run_state_manager();
	let greeter = WorkspaceService { tx, rx };

	Server::builder()
		.add_service(WorkspaceServer::new(greeter))
		.serve(addr)
		.await?;

	Ok(())
}
