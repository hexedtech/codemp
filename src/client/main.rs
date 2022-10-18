
mod nvim;
pub mod dispatcher;

use dispatcher::Dispatcher;

#[tokio::main]
async fn main() -> Result<(), Box<(dyn std::error::Error + 'static)>> {

	let dispatcher = Dispatcher::connect("http://[::1]:50051".into()).await.unwrap();

	#[cfg(feature = "nvim")]
	crate::nvim::run_nvim_client(dispatcher).await?;

	Ok(())
}
