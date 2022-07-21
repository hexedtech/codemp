use rmpv::Value;

use tokio::io::Stdout;

use nvim_rs::{compat::tokio::Compat, Handler, Neovim};
use nvim_rs::create::tokio::new_parent;
use tonic::transport::Channel;

use crate::proto::{SessionRequest, workspace_client::WorkspaceClient};

#[derive(Clone)]
pub struct NeovimHandler {
	go: bool,
	client: WorkspaceClient<Channel>,
}

impl NeovimHandler {
	pub fn new(client: WorkspaceClient<Channel>) -> Self {
		NeovimHandler { go: true, client }
	}

	async fn live_edit_worker(&self) {
		while self.go {

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
				let buf = neovim.get_current_buf().await.unwrap();
				let content = buf.get_lines(0, buf.line_count().await.unwrap(), false).await.unwrap().join("\n");
				let request = tonic::Request::new(SessionRequest {
					session_key: args[0].to_string(), content: Some(content),
				});
				let mut c = self.client.clone();
				let resp = c.create(request).await.unwrap().into_inner();
				if resp.accepted {
					Ok(Value::from(resp.session_key))
				} else {
					Err(Value::from("[!] rejected"))
				}
			},
			"sync" => {
				if args.len() < 1 {
					return Err(Value::from("[!] no session key"));
				}
				let buf = neovim.get_current_buf().await.unwrap();
				let request = tonic::Request::new(SessionRequest {
					session_key: args[0].to_string(), content: None,
				});
				let mut c = self.client.clone();
				let resp = c.sync(request).await.unwrap().into_inner();
				if let Some(content) = resp.content {
					buf.set_lines(
						0,
						buf.line_count().await.unwrap(),
						false,
						content.split("\n").map(|s| s.to_string()).collect()
					).await.unwrap();
					Ok(Value::from(""))
				} else {
					Err(Value::from("[!] no content"))
				}
			},
			"leave" => {
				if args.len() < 1 {
					return Err(Value::from("[!] no session key"));
				}
				let request = tonic::Request::new(SessionRequest {
					session_key: args[0].to_string(), content: None,
				});
				let mut c = self.client.clone();
				let resp = c.leave(request).await.unwrap().into_inner();
				if resp.accepted {
					Ok(Value::from(format!("closed session #{}", resp.session_key)))
				} else {
					Err(Value::from("[!] could not close session"))
				}
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
		_args: Vec<Value>,
		_neovim: Neovim<Compat<Stdout>>,
	) {
	match name.as_ref() {
			"insert" => {},
			"tick" => eprintln!("tock"),
			_ => eprintln!("[!] unexpected notify",)
		}
	}
}

pub async fn run_nvim_client(c: WorkspaceClient<Channel>) -> Result<(), Box<dyn std::error::Error + 'static>> {
	let handler: NeovimHandler = NeovimHandler::new(c);
	let (_nvim, io_handler) = new_parent(handler).await;

	// Any error should probably be logged, as stderr is not visible to users.
	match io_handler.await {
		Err(err) => eprintln!("Error joining IO loop: {:?}", err),
		Ok(Err(err)) => eprintln!("Process ended with error: {:?}", err),
		Ok(Ok(())) => eprintln!("Finished"),
	}

	Ok(())
}
