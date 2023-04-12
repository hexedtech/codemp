use std::{net::TcpStream, sync::Mutex};

use codemp::client::CodempClient;
use codemp::proto::buffer_client::BufferClient;
use rmpv::Value;


use tokio::io::Stdout;
use clap::Parser;

use nvim_rs::{compat::tokio::Compat, create::tokio as create, Handler, Neovim};
use tonic::async_trait;
use tracing::{error, warn, debug, info};

#[derive(Clone)]
struct NeovimHandler {
	client: CodempClient,
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

#[async_trait]
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
				let pos = default_zero_number(&args, 2) as u64;
				let mut c = self.client.clone();
				match c.insert(path, txt, pos).await {
					Ok(res) => {
						match res {
							true => Ok(Value::Nil),
							false => Err(Value::from("rejected")),
						}
					},
					Err(e) => Err(Value::from(format!("could not send insert: {}", e))),
				}
			},

			"delete" => {
				if args.len() < 3 {
					return Err(Value::from("not enough arguments"));
				}
				let path = default_empty_str(&args, 0);
				let pos = default_zero_number(&args, 1) as u64;
				let count = default_zero_number(&args, 2) as u64;

				let mut c = self.client.clone();
				match c.delete(path, pos, count).await {
					Ok(res) => match res {
						true => Ok(Value::Nil),
						false => Err(Value::from("rejected")),
					},
					Err(e) => Err(Value::from(format!("could not send insert: {}", e))),
				}
			},

			"replace" => {
				if args.len() < 2 {
					return Err(Value::from("not enough arguments"));
				}
				let path = default_empty_str(&args, 0);
				let txt = default_empty_str(&args, 1);

				let mut c = self.client.clone();
				match c.replace(path, txt).await {
					Ok(res) => match res {
						true => Ok(Value::Nil),
						false => Err(Value::from("rejected")),
					},
					Err(e) => Err(Value::from(format!("could not send replace: {}", e))),
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

				let buf = buffer.clone();
				match c.attach(path, move |x| {
					let lines : Vec<String> = x.split("\n").map(|x| x.to_string()).collect();
					let b = buf.clone();
					tokio::spawn(async move {
						if let Err(e) = b.set_lines(0, -1, false, lines).await {
							error!("could not update buffer: {}", e);
						}
					});
				}).await {
					Err(e) => Err(Value::from(format!("could not attach to stream: {}", e))),
					Ok(content) => {
						let lines : Vec<String> = content.split("\n").map(|x| x.to_string()).collect();
						if let Err(e) = buffer.set_lines(0, -1, false, lines).await {
							error!("could not update buffer: {}", e);
						}
						Ok(Value::Nil)
					},
				}
			},

			"detach" => {
				if args.len() < 1 {
					return Err(Value::from("no path given"));
				}
				let path = default_empty_str(&args, 0);
				let mut c = self.client.clone();
				c.detach(path);
				Ok(Value::Nil)
			},

			"listen" => {
				if args.len() < 1 {
					return Err(Value::from("no path given"));
				}
				let path = default_empty_str(&args, 0);
				let mut c = self.client.clone();

				let ns = nvim.create_namespace("Cursor").await
					.map_err(|e| Value::from(format!("could not create namespace: {}", e)))?;

				let buf = nvim.get_current_buf().await
					.map_err(|e| Value::from(format!("could not get current buf: {}", e)))?;
				
				match c.listen(path, move |cur| {
					let _b = buf.clone();
					tokio::spawn(async move {
						if let Err(e) = _b.clear_namespace(ns, 0, -1).await {
							error!("could not clear previous cursor highlight: {}", e);
						}
						if let Err(e) = _b.add_highlight(ns, "ErrorMsg", cur.row-1, cur.col, cur.col+1).await {
							error!("could not create highlight for cursor: {}", e);
						}
					});
				}).await {
					Ok(()) => Ok(Value::Nil),
					Err(e) => Err(Value::from(format!("could not listen cursors: {}", e))),
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
					Ok(()) => Ok(Value::Nil),
					Err(e) => Err(Value::from(format!("could not send cursor update: {}", e))),
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
	};

	let (_nvim, io_handler) = create::new_parent(handler).await;

	info!("++ codemp connected: {}", args.host);

	if let Err(e) = io_handler.await? {
		error!("worker stopped with error: {}", e);
	}

	Ok(())
}
