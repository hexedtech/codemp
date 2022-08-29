//! # codemp Server
//!
//! The codemp server itself, in charge of handling the global state, merging operations from
//!  all clients and synching everyone's cursor.
//!

pub mod actor;
pub mod events;
pub mod service;

use std::sync::Arc;

use tracing::info;

use tonic::transport::Server;

use crate::{
	actor::state::StateManager,
	service::{buffer::BufferService, workspace::WorkspaceService},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	tracing_subscriber::fmt::init();

	let addr = "[::1]:50051".parse()?;

	let state = Arc::new(StateManager::new());

	info!("Starting server");

	Server::builder()
		.add_service(WorkspaceService::server(state.clone()))
		.add_service(BufferService::server(state.clone()))
		.serve(addr)
		.await?;

	Ok(())
}
