use std::sync::Arc;
use std::{net::TcpStream, sync::Mutex, collections::BTreeMap};

use codemp::operation::{OperationController, OperationFactory, OperationProcessor};
use codemp::client::CodempClient;
use codemp::proto::buffer_client::BufferClient;
use rmpv::Value;


use tokio::io::Stdout;
use clap::Parser;

use nvim_rs::{compat::tokio::Compat, create::tokio as create, Handler, Neovim};
use tracing::{error, warn, debug, info};

#[derive(Clone)]
struct NeovimHandler {
	client: CodempClient,
	factories: Arc<Mutex<BTreeMap<String, Arc<OperationController>>>>,
}

fn nullable_optional_str(args: &Vec<Value>, index: usize) -> Option<String> {
	Some(args.get(index)?.as_str()?.to_string())
}

fn default_empty_str(args: &Vec<Value>, index: usize) -> String {
	nullable_optional_str(args, index).unwrap_or("".into())
}

fn nullable_optional_number(args: &Vec<Value>, index: usize) -> Option<i64> {
	Some(args.get(index)?.as_i64()?)
}

fn default_zero_number(args: &Vec<Value>, index: usize) -> i64 {
	nullable_optional_number(args, index).unwrap_or(0)
}

impl NeovimHandler {
	fn buffer_controller(&self, path: &String) -> Option<Arc<OperationController>> {
		Some(self.factories.lock().unwrap().get(path)?.clone())
	}
}

#[tonic::async_trait]
impl Handler for NeovimHandler {
	type Writer = Compat<Stdout>;

	async fn handle_request(
		&self,
		name: String,
		args: Vec<Value>,
		nvim: Neovim<Compat<Stdout>>,
	) -> Result<Value, Value> {
		debug!("processing '{}' - {:?}", name, args);
		match name.as_ref() {
			"ping" => Ok(Value::from("pong")),

			"create" => {
				if args.len() < 1 {
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
				let mut pos = default_zero_number(&args, 2) as i64;
				
				if pos <= 0 { pos = 0 } // TODO wtf vim??

				match self.buffer_controller(&path) {
					None => Err(Value::from("no controller for given path")),
					Some(controller) => {
						match controller.apply(controller.insert(&txt, pos as u64)).await {
							Err(e) => Err(Value::from(format!("could not send insert: {}", e))),
							Ok(_res) => Ok(Value::Nil),
						}
					}
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
					Some(controller) => match controller.apply(controller.delete(pos, count)).await {
						Err(e) => Err(Value::from(format!("could not send delete: {}", e))),
						Ok(_res) => Ok(Value::Nil),
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
					Some(controller) => match controller.apply(controller.replace(&txt)).await {
						Err(e) => Err(Value::from(format!("could not send replace: {}", e))),
						Ok(_res) => Ok(Value::Nil),
					}
				}
			},

			"attach" => {
				if args.len() < 1 {
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
						let _controller = controller.clone();
						let lines : Vec<String> = _controller.content().split("\n").map(|x| x.to_string()).collect();
						match buffer.set_lines(0, -1, false, lines).await {
							Err(e) => Err(Value::from(format!("could not sync buffer: {}", e))),
							Ok(()) => {
								tokio::spawn(async move {
									loop {
										if !_controller.run() { break debug!("buffer updater clean exit") }
										let _span = _controller.wait().await;
										// TODO only change lines affected!
										let lines : Vec<String> = _controller.content().split("\n").map(|x| x.to_string()).collect();
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
				if args.len() < 1 {
					return Err(Value::from("no path given"));
				}
				let path = default_empty_str(&args, 0);
				match self.buffer_controller(&path) {
					None => Err(Value::from("no controller for given path")),
					Some(controller) => Ok(Value::from(controller.stop())),
				}
			},

			"listen" => {
				if args.len() < 1 {
					return Err(Value::from("no path given"));
				}
				let path = default_empty_str(&args, 0);
				let controller = match self.buffer_controller(&path) {
					None => return Err(Value::from("no controller for given path")),
					Some(c) => c,
				};

				let ns = nvim.create_namespace("Cursor").await
					.map_err(|e| Value::from(format!("could not create namespace: {}", e)))?;

				let buf = nvim.get_current_buf().await
					.map_err(|e| Value::from(format!("could not get current buf: {}", e)))?;
				
				let mut c = self.client.clone();
				match c.listen().await {
					Err(e) => Err(Value::from(format!("could not listen cursors: {}", e))),
					Ok(cursor) => {
						let mut sub = cursor.sub();
						debug!("spawning cursor processing worker");
						tokio::spawn(async move {
							loop {
								if !controller.run() { break debug!("cursor worker clean exit") }
								match sub.recv().await {
									Err(e) => break error!("error receiving cursor update from controller: {}", e),
									Ok((_usr, cur)) => {
										if let Err(e) = buf.clear_namespace(ns, 0, -1).await {
											error!("could not clear previous cursor highlight: {}", e);
										}
										if let Err(e) = buf.add_highlight(ns, "ErrorMsg", cur.start.row-1, cur.start.col, cur.start.col+1).await {
											error!("could not create highlight for cursor: {}", e);
										}
									}
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
				let row = default_zero_number(&args, 1);
				let col = default_zero_number(&args, 2);

				let mut c = self.client.clone();
				match c.cursor(path, row, col).await {
					Ok(_) => Ok(Value::Nil),
					Err(e) => Err(Value:: from(format!("could not update cursor: {}", e))),
				}
			},

			_ => Err(Value::from("unimplemented")),
		}
	}

	async fn handle_notify(
		&self,
		_name: String,
		_args: Vec<Value>,
		_nvim: Neovim<Compat<Stdout>>,
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
	};

	let (_nvim, io_handler) = create::new_parent(handler).await;

	info!("++ codemp connected: {}", args.host);

	if let Err(e) = io_handler.await? {
		error!("worker stopped with error: {}", e);
	}

	Ok(())
}
