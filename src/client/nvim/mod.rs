use std::sync::Arc;

use rmpv::Value;

use tokio::io::Stdout;

use nvim_rs::{compat::tokio::Compat, Handler, Neovim};
use nvim_rs::create::tokio::new_parent;
use tokio::sync::{mpsc, Mutex};

use crate::dispatcher::{Dispatcher, proto::CursorUpdate};

#[derive(Clone)]
pub struct NeovimHandler {
	dispatcher: Dispatcher,
	sink: Arc<Mutex<Option<mpsc::Sender<CursorUpdate>>>>,
}

impl NeovimHandler {
	pub fn new(dispatcher: Dispatcher) -> Self {
		NeovimHandler {
			dispatcher,
			sink: Arc::new(Mutex::new(None)),
		}
	}
}

#[tonic::async_trait]
impl Handler for NeovimHandler {
	type Writer = Compat<Stdout>;

	async fn handle_request(
		&self,
		name: String,
		args: Vec<Value>,
		neovim: Neovim<Compat<Stdout>>,
	) -> Result<Value, Value> {
		match name.as_ref() {
			"ping" => Ok(Value::from("pong")),
			"create" => {
				if args.len() < 1 {
					return Err(Value::from("[!] no session key"));
				}
				let res = self.dispatcher.create_workspace(args[0].to_string())
					.await
					.map_err(|e| Value::from(e.to_string()))?
					.into_inner();

				Ok(res.session_key.into())
			},
			"join" => {
				if args.len() < 1 {
					return Err(Value::from("[!] no session key"));
				}

				self.dispatcher.join_workspace(
					args[0].as_str().unwrap().to_string(), // TODO throw err if it's not a string?
				).await.map_err(|e| Value::from(e.to_string()))?;

				Ok("OK".into())
			},
			"cursor-start" => {
				if args.len() < 1 {
					return Err(Value::from("[!] no session key"));
				}
				let (tx, stream) = mpsc::channel(50);
				let mut rx = self.dispatcher.start_cursor_worker(
					args[0].as_str().unwrap().to_string(), stream
				).await.map_err(|e| Value::from(e.to_string()))?;
				let sink = self.sink.clone();
				sink.lock().await.replace(tx);
				let _worker = tokio::spawn(async move {
					let mut col : i64;
					let mut row : i64 = 0;
					let ns = neovim.create_namespace("Cursor").await.unwrap();
					while let Some(update) = rx.recv().await {
						neovim.exec_lua(format!("print('{:?}')", update).as_str(), vec![]).await.unwrap();
						let buf = neovim.get_current_buf().await.unwrap();
						buf.clear_namespace(ns, 0, -1).await.unwrap();
						row = update.row as i64;
						col = update.col as i64;
						buf.add_highlight(ns, "ErrorMsg", row-1, col-1, col).await.unwrap();
					}
					sink.lock().await.take();
				});
				Ok("OK".into())
			},
			_ => {
				eprintln!("[!] unexpected call");
				Ok(Value::from(""))
			},
		}
	}

	async fn handle_notify(
		&self,
		name: String,
		args: Vec<Value>,
		_neovim: Neovim<Compat<Stdout>>,
	) {
	match name.as_ref() {
			"insert" => {},
			"cursor" => {
				if args.len() >= 3 {
					if let Some(sink) = self.sink.lock().await.as_ref() {
						sink.send(CursorUpdate {
							buffer: args[0].as_i64().unwrap(),
							row: args[1].as_i64().unwrap(),
							col: args[2].as_i64().unwrap(),
							username: "root".into()
						}).await.unwrap();
					}
				}
			},
			"tick" => eprintln!("tock"),
			_ => eprintln!("[!] unexpected notify",)
		}
	}
}

pub async fn run_nvim_client(dispatcher: Dispatcher) -> Result<(), Box<dyn std::error::Error + 'static>> {
	let handler: NeovimHandler = NeovimHandler::new(dispatcher);
	let (_nvim, io_handler) = new_parent(handler).await;

	// Any error should probably be logged, as stderr is not visible to users.
	match io_handler.await {
		Err(err) => eprintln!("Error joining IO loop: {:?}", err),
		Ok(Err(err)) => eprintln!("Process ended with error: {:?}", err),
		Ok(Ok(())) => eprintln!("Finished"),
	}

	Ok(())
}
