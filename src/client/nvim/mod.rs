use rmpv::Value;

use tokio::io::Stdout;
use tokio::sync::mpsc;

use nvim_rs::{compat::tokio::Compat, Handler, Neovim};

use crate::manager::Command;

#[derive(Clone)]
pub struct NeovimHandler {
	tx: mpsc::Sender<Command>,
}

impl NeovimHandler {
	pub async fn new(tx: mpsc::Sender<Command>) -> Result<Self, tonic::transport::Error> {
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
				let (cmd, rx) = Command::create_session_cmd("asd".to_string()); 
				self.tx.send(cmd).await.unwrap();
				let resp = rx.await.unwrap();
				Ok(Value::from(format!("{:?}", resp)))
			},
			"buffer" => {
				let buf = _neovim.create_buf(true, false).await.unwrap();
				buf.set_lines(0, 1, false, vec!["codeMP".to_string()]).await.unwrap();
				_neovim.set_current_buf(&buf).await.unwrap();
				Ok(Value::from("ok"))
			}
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
