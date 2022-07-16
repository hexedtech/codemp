mod nvim;

pub mod proto { tonic::include_proto!("workspace"); }
use proto::workspace_client::WorkspaceClient;

#[tokio::main]
async fn main() -> Result<(), Box<(dyn std::error::Error + 'static)>> {
	let client = WorkspaceClient::connect("http://[::1]:50051").await?;

	#[cfg(feature = "nvim")]
	crate::nvim::run_nvim_client(client).await.unwrap();

	Ok(())
}
