//! # codemp Server
//!
//! The codemp server itself, in charge of handling the global state, merging operations from
//!  all clients and synching everyone's cursor.
//!

use clap::Parser;
use codemp::proto::buffer_server::BufferServer;
use codemp::proto::cursor_server::CursorServer;
use tracing::info;
use tonic::transport::Server;

mod buffer;
mod cursor;

use crate::buffer::service::BufferService;
use crate::cursor::service::CursorService;

#[derive(Parser, Debug)]
struct CliArgs {

	/// address to listen on
	#[arg(long, default_value = "[::1]:50051")]
	host: String,

	/// enable debug log level
	#[arg(long, default_value_t = false)]
	debug: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let args = CliArgs::parse();

	tracing_subscriber::fmt()
		.with_writer(std::io::stdout)
		.with_max_level(if args.debug { tracing::Level::DEBUG } else { tracing::Level::INFO })
		.init();

	info!(">> codemp server");
	info!("binding on {}", args.host);

	Server::builder()
		.add_service(BufferServer::new(BufferService::default()))
		.add_service(CursorServer::new(CursorService::default()))
		.serve(args.host.parse()?)
		.await?;

	Ok(())
}
