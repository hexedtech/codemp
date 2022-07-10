use rmpv::Value;

use tokio::io::Stdout;
use tokio::sync::mpsc;

use nvim_rs::{compat::tokio::Compat, create::tokio::new_parent, rpc::IntoVal, Handler, Neovim};
use tonic::transport::Channel;

use crate::manager::proto_core::{session_client::SessionClient, SessionRequest};


#[derive(Clone)]
pub struct NeovimHandler {
	tx: mpsc::Sender<i32>,
}

impl NeovimHandler {
	pub async fn new(tx: mpsc::Sender<i32>) -> Result<Self, tonic::transport::Error> {
		Ok(NeovimHandler { tx })
	}
}

#[tonic::async_trait]
impl Handler for NeovimHandler {
	type Writer = Compat<Stdout>;

	async fn handle_request(
		&self,
		name: String,
		_args: Vec<Value>,
		_neovim: Neovim<Compat<Stdout>>,
	) -> Result<Value, Value> {
		match name.as_ref() {
			"ping" => Ok(Value::from("pong")),
			"rpc" => {
				self.tx.send(0).await.unwrap();
				// let request = tonic::Request::new(SessionRequest {session_id: 0});
				// let response = self.client.create(request).await.unwrap();
				Ok(Value::from("sent"))
			},
			_ => unimplemented!(),
		}
	}
}

pub async fn run_nvim_plugin(tx: mpsc::Sender<i32>) -> Result<(), Box<(dyn std::error::Error + 'static)>> {
	let handler: NeovimHandler = NeovimHandler::new(tx).await?;
	let (nvim, io_handler) = new_parent(handler).await;
	let curbuf = nvim.get_current_buf().await.unwrap();

	let mut envargs = std::env::args();
	let _ = envargs.next();
	let testfile = envargs.next().unwrap();

	std::fs::write(testfile, &format!("{:?}", curbuf.into_val())).unwrap();
	

	// Any error should probably be logged, as stderr is not visible to users.
	match io_handler.await {
		Err(	err) => eprintln!("Error joining IO loop: '{}'", joinerr),
		Ok(Err(err)) => {
			if !err.is_reader_error() {
				// One last try, since there wasn't an error with writing to the
				// stream
				nvim
					.err_writeln(&format!("Error: '{}'", err))
					.await
					.unwrap_or_else(|e| {
						// We could inspect this error to see what was happening, and
						// maybe retry, but at this point it's probably best
						// to assume the worst and print a friendly and
						// supportive message to our users
						eprintln!("Well, dang... '{}'", e);
					});
			}

			if !err.is_channel_closed() {
				// Closed channel usually means neovim quit itself, or this plugin was
				// told to quit by closing the channel, so it's not always an error
				// condition.
				eprintln!("Error: '{}'", err);

				// let mut source = err.source();

				// while let Some(e) = source {
				// 	eprintln!("Caused by: '{}'", e);
				// 	source = e.source();
				// }
			}
		}
		Ok(Ok(())) => {}
	}

	Ok(())
}
