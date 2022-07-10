pub mod manager;
mod nvim;

use tokio::sync::mpsc;
use nvim_rs::{compat::tokio::Compat, create::tokio::new_parent, rpc::IntoVal, Handler, Neovim, Value};

use manager::ConnectionManager;
use nvim::NeovimHandler;


#[tokio::main]
async fn main() -> Result<(), Box<(dyn std::error::Error + 'static)>> {
	let (tx, rx) = mpsc::channel(32);
	let mut mngr = ConnectionManager::new("http://[::1]:50051".to_string(), rx).await?;
	tokio::spawn(async move {
		mngr.process_packets().await
	});

	let handler: NeovimHandler = NeovimHandler::new(tx).await?;
	let (nvim, io_handler) = new_parent(handler).await;

	nvim.call(":echo", vec![Value::from("***REMOVED***")]).await.unwrap().unwrap();

	// Any error should probably be logged, as stderr is not visible to users.
	match io_handler.await {
		Err(err) => eprintln!("Error joining IO loop: {:?}", err),
		Ok(Err(err)) => eprintln!("Process ended with error: {:?}", err),
		Ok(Ok(())) => println!("Finished"),
	}

	Ok(())
}
