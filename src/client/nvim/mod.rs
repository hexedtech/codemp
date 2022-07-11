use rmpv::Value;

use tokio::io::Stdout;
use tokio::sync::mpsc;

use nvim_rs::{compat::tokio::Compat, Handler, Neovim};

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
				eprintln!("Got 'rpc' from vim");
				self.tx.send(0).await.unwrap();
				// let request = tonic::Request::new(SessionRequest {session_id: 0});
				// let response = self.client.create(request).await.unwrap();
				Ok(Value::from("sent"))
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
