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

fn nullable_optional_number(args: &Vec<Value>, index: usize) -> Option<u64> {
	Some(args.get(index)?.as_u64()?)
}

fn default_zero_number(args: &Vec<Value>, index: usize) -> u64 {
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

			"error" => Err(Value::from("user-requested error")),

			"create" => {
				if args.len() < 1 {
					return Err(Value::from("no path given"));
				}
				let path = default_empty_str(&args, 0);
				let content = nullable_optional_str(&args, 1);
				let mut c = self.client.clone();
				match c.create(path, content).await {
					Ok(r) => match r {
						true => Ok(Value::from("accepted")),
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
				let pos = default_zero_number(&args, 2);
				let mut c = self.client.clone();
				info!("correctly parsed arguments: {} - {} - {}", path, txt, pos);
				match c.insert(path, txt, pos).await {
					Ok(res) => {
						info!("RPC 'insert' completed");
						match res {
							true => Ok(Value::from("accepted")),
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
				let pos = default_zero_number(&args, 1);
				let count = default_zero_number(&args, 2);

				let mut c = self.client.clone();
				match c.delete(path, pos, count).await {
					Ok(res) => match res {
						true => Ok(Value::from("accepted")),
						false => Err(Value::from("rejected")),
					},
					Err(e) => Err(Value::from(format!("could not send insert: {}", e))),
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
						Ok(Value::from("spawned worker"))
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
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let args = CliArgs::parse();

	let sub = tracing_subscriber::fmt();
	match TcpStream::connect("127.0.0.1:6969") { // TODO get rid of this
		Ok(stream) => {
			sub.with_writer(Mutex::new(stream))
				.with_max_level(if args.debug { tracing::Level::DEBUG } else { tracing::Level::INFO })
				.init();
		},
		Err(_) => {
			sub
				.compact()
				.without_time()
				.with_ansi(false)
				.with_writer(std::io::stderr)
				.with_max_level(if args.debug { tracing::Level::DEBUG } else { tracing::Level::INFO })
				.init();
		},
	}

	let client = BufferClient::connect(args.host).await?;
	debug!("client connected");

	let handler: NeovimHandler = NeovimHandler {
		client: client.into(),
	};

	let (nvim, io_handler) = create::new_parent(handler).await;

	nvim.out_write("[*] codemp loaded").await?;

	if let Err(e) = io_handler.await? {
		error!("[!] worker stopped with error: {}", e);
	}

	Ok(())
}
