pub mod manager;
mod nvim;

use tokio::sync::mpsc;
use nvim_rs::{create::tokio::new_parent};

use manager::ConnectionManager;
use nvim::NeovimHandler;

#[tokio::main]
async fn main() -> Result<(), Box<(dyn std::error::Error + 'static)>> {
	let (tx, rx) = mpsc::channel(32);

	let handler: NeovimHandler = NeovimHandler::new(tx).await?;
	let (_nvim, io_handler) = new_parent(handler).await;

	// nvim.call(":echo", vec![Value::from("'***REMOVED***'")]).await.unwrap();
	let mut mngr = ConnectionManager::new("http://[::1]:50051".to_string(), rx).await?;
	let _task = tokio::spawn(async move {
		mngr.process_packets().await
	});

	// Any error should probably be logged, as stderr is not visible to users.
	match io_handler.await {
		Err(err) => eprintln!("Error joining IO loop: {:?}", err),
		Ok(Err(err)) => eprintln!("Process ended with error: {:?}", err),
		Ok(Ok(())) => eprintln!("Finished"),
	}

	Ok(())
}
