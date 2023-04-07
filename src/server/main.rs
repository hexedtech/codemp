//! # codemp Server
//!
//! The codemp server itself, in charge of handling the global state, merging operations from
//!  all clients and synching everyone's cursor.
//!

mod buffer;

use tracing::info;

use tonic::transport::Server;

use crate::buffer::service::BufferService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	tracing_subscriber::fmt::init();

	let addr = "[::1]:50051".parse()?;

	info!("Starting server");

	Server::builder()
		.add_service(BufferService::new().server())
		.serve(addr)
		.await?;

	Ok(())
}
