//! # codemp Server
//!
//! The codemp server itself, in charge of handling the global state, merging operations from
//!  all clients and synching everyone's cursor.
//!

pub mod actor;
pub mod service;

use std::sync::Arc;

use tracing::info;

use tonic::transport::Server;

use crate::{
	actor::state::StateManager,
	service::{buffer::BufferService, workspace::WorkspaceService, session::SessionService},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	tracing_subscriber::fmt::init();

	let addr = "[::1]:50051".parse()?;

	let state = Arc::new(StateManager::new());

	info!("Starting server");

	Server::builder()
		.add_service(SessionService::new(state.clone()).server())
		.add_service(WorkspaceService::new(state.clone()).server())
		.add_service(BufferService::new(state.clone()).server())
		.serve(addr)
		.await?;

	Ok(())
}
