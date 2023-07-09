use std::sync::Arc;
use std::{net::TcpStream, sync::Mutex, collections::BTreeMap};

use codemp::client::CodempClient;
use codemp::controller::buffer::{OperationControllerHandle, OperationControllerSubscriber};
use codemp::controller::cursor::{CursorControllerHandle, CursorSubscriber};
use codemp::factory::OperationFactory;
use codemp::proto::buffer_client::BufferClient;
use codemp::tokio;

use rmpv::Value;
use clap::Parser;

use nvim_rs::{compat::tokio::Compat, create::tokio as create, Handler, Neovim};
use tracing::{error, warn, debug, info};

#[derive(Clone)]
struct NeovimHandler {
	client: CodempClient,
	factories: Arc<Mutex<BTreeMap<String, OperationControllerHandle>>>,
	cursors: Arc<Mutex<BTreeMap<String, CursorControllerHandle>>>,
}

fn nullable_optional_str(args: &[Value], index: usize) -> Option<String> {
	Some(args.get(index)?.as_str()?.to_string())
}

fn default_empty_str(args: &[Value], index: usize) -> String {
	nullable_optional_str(args, index).unwrap_or("".into())
}

fn nullable_optional_number(args: &[Value], index: usize) -> Option<i64> {
	args.get(index)?.as_i64()
}

fn default_zero_number(args: &[Value], index: usize) -> i64 {
	nullable_optional_number(args, index).unwrap_or(0)
}

impl NeovimHandler {
	fn buffer_controller(&self, path: &String) -> Option<OperationControllerHandle> {
		Some(self.factories.lock().unwrap().get(path)?.clone())
	}

	fn cursor_controller(&self, path: &String) -> Option<CursorControllerHandle> {
		Some(self.cursors.lock().unwrap().get(path)?.clone())
	}
}

#[async_trait::async_trait]
impl Handler for NeovimHandler {
	type Writer = Compat<tokio::io::Stdout>;

	async fn handle_request(
		&self,
		name: String,
		args: Vec<Value>,
		nvim: Neovim<Compat<tokio::io::Stdout>>,
	) -> Result<Value, Value> {
		debug!("processing '{}' - {:?}", name, args);
		match name.as_ref() {
			"ping" => Ok(Value::from("pong")),

			"create" => {
				if args.is_empty() {
					return Err(Value::from("no path given"));
				}
				let path = default_empty_str(&args, 0);
				let content = nullable_optional_str(&args, 1);
				let mut c = self.client.clone();
				match c.create(path, content).await {
					Ok(r) => match r {
						true => Ok(Value::Nil),
						false => Err(Value::from("rejected")),
					},
					Err(e) => Err(Value::from(format!("could not create buffer: {}", e))),
				}
			},

			"insert" => {
				if args.len() < 3 {
					return Err(Value::from("not enough arguments"));
				}
				let path = default_empty_str(&args, 0);
				let txt = default_empty_str(&args, 1);
				let mut pos = default_zero_number(&args, 2);
				
				if pos <= 0 { pos = 0 } // TODO wtf vim??

				match self.buffer_controller(&path) {
					None => Err(Value::from("no controller for given path")),
					Some(controller) => {
						controller.apply(controller.insert(&txt, pos as u64)).await;
						Ok(Value::Nil)
					},
				}
			},

			"delete" => {
				if args.len() < 3 {
					return Err(Value::from("not enough arguments"));
				}
				let path = default_empty_str(&args, 0);
				let pos = default_zero_number(&args, 1) as u64;
				let count = default_zero_number(&args, 2) as u64;

				match self.buffer_controller(&path) {
					None => Err(Value::from("no controller for given path")),
					Some(controller) => {
						controller.apply(controller.delete(pos, count)).await;
						Ok(Value::Nil)
					}
				}
			},

			"replace" => {
				if args.len() < 2 {
					return Err(Value::from("not enough arguments"));
				}
				let path = default_empty_str(&args, 0);
				let txt = default_empty_str(&args, 1);

				match self.buffer_controller(&path) {
					None => Err(Value::from("no controller for given path")),
					Some(controller) => {
						if let Some(op) = controller.replace(&txt) {
							controller.apply(op).await;
						}
						Ok(Value::Nil)
					}
				}
			},

			"attach" => {
				if args.is_empty() {
					return Err(Value::from("no path given"));
				}
				let path = default_empty_str(&args, 0);
				let buffer = match nvim.get_current_buf().await {
					Ok(b) => b,
					Err(e) => return Err(Value::from(format!("could not get current buffer: {}", e))),
				};

				let mut c = self.client.clone();

				match c.attach(path.clone()).await {
					Err(e) => Err(Value::from(format!("could not attach to stream: {}", e))),
					Ok(controller) => {
						let mut _controller = controller.clone();
						let lines : Vec<String> = _controller.content().split('\n').map(|x| x.to_string()).collect();
						match buffer.set_lines(0, -1, false, lines).await {
							Err(e) => Err(Value::from(format!("could not sync buffer: {}", e))),
							Ok(()) => {
								tokio::spawn(async move {
									while let Some(_change) = _controller.poll().await {
										let lines : Vec<String> = _controller.content().split('\n').map(|x| x.to_string()).collect();
										// TODO only change lines affected!
										if let Err(e) = buffer.set_lines(0, -1, false, lines).await {
											error!("could not update buffer: {}", e);
										}
									}
								});
								self.factories.lock().unwrap().insert(path, controller);
								Ok(Value::Nil)
							}
						}
					},
				}
			},

			"detach" => {
				Err(Value::String("not implemented".into()))
				// if args.is_empty() {
				// 	return Err(Value::from("no path given"));
				// }
				// let path = default_empty_str(&args, 0);
				// match self.buffer_controller(&path) {
				// 	None => Err(Value::from("no controller for given path")),
				// 	Some(controller) => Ok(Value::from(controller.stop())),
				// }
			},

			"listen" => {
				if args.is_empty() {
					return Err(Value::from("no path given"));
				}
				let path = default_empty_str(&args, 0);

				let ns = nvim.create_namespace("Cursor").await
					.map_err(|e| Value::from(format!("could not create namespace: {}", e)))?;

				let buf = nvim.get_current_buf().await
					.map_err(|e| Value::from(format!("could not get current buf: {}", e)))?;
				
				let mut c = self.client.clone();
				match c.listen().await {
					Err(e) => Err(Value::from(format!("could not listen cursors: {}", e))),
					Ok(mut cursor) => {
						self.cursors.lock().unwrap().insert(path, cursor.clone());
						debug!("spawning cursor processing worker");
						tokio::spawn(async move {
							while let Some(cur) = cursor.poll().await {
								if let Err(e) = buf.clear_namespace(ns, 0, -1).await {
									error!("could not clear previous cursor highlight: {}", e);
								}
								let start = cur.start();
								let end = cur.end();
								let end_col = if start.row == end.row {
									end.col
								} else {
									0 // TODO what the fuck
								};
								if let Err(e) = buf.add_highlight(
									ns, "ErrorMsg",
									start.row as i64 - 1,
									start.col as i64,
									end_col as i64
								).await {
									error!("could not create highlight for cursor: {}", e);
								}
							}
							if let Err(e) = buf.clear_namespace(ns, 0, -1).await {
								error!("could not clear previous cursor highlight: {}", e);
							}
						});
						Ok(Value::Nil)
					},
				}
			},

			"cursor" => {
				if args.len() < 3 {
					return Err(Value::from("not enough args"));
				}
				let path = default_empty_str(&args, 0);
				let row = default_zero_number(&args, 1) as i32;
				let col = default_zero_number(&args, 2) as i32;
				let row_end = default_zero_number(&args, 3) as i32;
				let col_end = default_zero_number(&args, 4) as i32;

				match self.cursor_controller(&path) {
					None => Err(Value::from("no path given")),
					Some(cur) => {
						cur.send(&path, (row, col).into(), (row_end, col_end).into()).await;
						Ok(Value::Nil)
					}
				}
			},

			_ => Err(Value::from("unimplemented")),
		}
	}

	async fn handle_notify(
		&self,
		_name: String,
		_args: Vec<Value>,
		_nvim: Neovim<Compat<tokio::io::Stdout>>,
	) {
		warn!("notify not handled");
	}
}

#[derive(Parser, Debug)]
struct CliArgs {
	/// server host to connect to
	#[arg(long, default_value = "http://[::1]:50051")]
	host: String,

	/// show debug level logs
	#[arg(long, default_value_t = false)]
	debug: bool,

	/// dump raw tracing logs into this TCP host
	#[arg(long)]
	remote_debug: Option<String>,
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let args = CliArgs::parse();

	match args.remote_debug {
		Some(host) =>
			tracing_subscriber::fmt()
				.with_writer(Mutex::new(TcpStream::connect(host)?))
				.with_max_level(if args.debug { tracing::Level::DEBUG } else { tracing::Level::INFO })
				.init(),

		None =>
			tracing_subscriber::fmt()
				.compact()
				.without_time()
				.with_ansi(false)
				.with_writer(std::io::stderr)
				.with_max_level(if args.debug { tracing::Level::DEBUG } else { tracing::Level::INFO })
				.init(),
	}

	let client = BufferClient::connect(args.host.clone()).await?;

	let handler: NeovimHandler = NeovimHandler {
		client: client.into(),
		factories: Arc::new(Mutex::new(BTreeMap::new())),
		cursors: Arc::new(Mutex::new(BTreeMap::new())),
	};

	let (_nvim, io_handler) = create::new_parent(handler).await;

	info!("++ codemp connected: {}", args.host);

	if let Err(e) = io_handler.await? {
		error!("worker stopped with error: {}", e);
	}

	Ok(())
}
