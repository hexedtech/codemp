use codemp::client::CodempClient;
use codemp::proto::buffer_client::BufferClient;
use rmpv::Value;


use tokio::io::Stdout;
use clap::Parser;

use nvim_rs::{compat::tokio::Compat, create::tokio as create, Handler, Neovim};
use tonic::async_trait;
use tracing::{error, warn, debug};

#[derive(Clone)]
struct NeovimHandler {
	client: CodempClient,
}

fn nullable_optional_str(args: &Vec<Value>, index: usize) -> Option<String> {
	Some(args.get(index)?.as_str()?.to_string())
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
		match name.as_ref() {
			"ping" => Ok(Value::from("pong")),

			"dump" => Ok(Value::from(self.client.content())),

			"create" => {
				if args.len() < 1 {
					return Err(Value::from("no path given"));
				}
				let path = args.get(0).unwrap().as_str().unwrap().into();
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
				let path = args.get(0).unwrap().as_str().unwrap().into();
				let txt = args.get(1).unwrap().as_str().unwrap().into();
				let pos = args.get(2).unwrap().as_u64().unwrap();

				let mut c = self.client.clone();
				match c.insert(path, txt, pos).await {
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
				let path = args.get(0).unwrap().as_str().unwrap().into();
				let buf = nvim.get_current_buf().await.unwrap();
				let mut c = self.client.clone();

				match c.attach(path, move |x| {
					let lines : Vec<String> = x.split("\n").map(|x| x.to_string()).collect();
					let b = buf.clone();
					tokio::spawn(async move {
						if let Err(e) = b.set_lines(0, lines.len() as i64, false, lines).await {
							error!("could not update buffer: {}", e);
						}
					});
				}).await {
					Ok(()) => Ok(Value::from("spawned worker")),
					Err(e) => Err(Value::from(format!("could not attach to stream: {}", e))),
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
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let args = CliArgs::parse();

	tracing_subscriber::fmt()
		.compact()
		.without_time()
		.with_ansi(false)
		.with_writer(std::io::stderr)
		.with_max_level(if args.debug { tracing::Level::DEBUG } else { tracing::Level::INFO })
		.init();

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
