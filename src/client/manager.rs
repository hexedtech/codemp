pub mod proto_core {
	tonic::include_proto!("session");
}

use tonic::transport::Channel;

use proto_core::workspace_client::WorkspaceClient;
use proto_core::SessionRequest;

use tokio::sync::{mpsc, oneshot};

use self::proto_core::SessionResponse;

#[derive(Debug)]
pub enum Command {
	CreateSession {
		key: String,
		resp: oneshot::Sender<SessionResponse>,
	},
	JoinSession {
		key: String,
		resp: oneshot::Sender<SessionResponse>,
	},
}

impl Command {
	pub fn create_session_cmd(key: String) -> (Command, oneshot::Receiver<SessionResponse>) {
		let (resp, x) = oneshot::channel();
		( Command::CreateSession { key, resp }, x )
	}
}

#[derive(Debug)]
pub struct ConnectionManager {
	client: WorkspaceClient<Channel>,
	rx: mpsc::Receiver<Command>
}


impl ConnectionManager {
	pub async fn new(addr:String, outbound:mpsc::Receiver<Command>) -> Result<Self, Box<dyn std::error::Error>> {
		Ok(ConnectionManager {
			client: WorkspaceClient::connect(addr).await?,
			rx: outbound
		})
	}

	pub async fn process_packets(&mut self) {
		{
			let request = tonic::Request::new(SessionRequest {
				session_id: -1,
				session_key: "INIT".to_string(),
			});
			let response = self.client.create(request).await.unwrap();
			eprintln!("RESPONSE={:?}", response);
		}
		loop {
			if let Some(cmd) = self.rx.recv().await {
				match cmd {
					Command::CreateSession { key, resp } => {
						let request = tonic::Request::new(SessionRequest {session_id: 1, session_key: key});
						let response = self.client.create(request).await.unwrap();
						resp.send(response.into_inner()).unwrap();
					},
					_ => eprintln!("[!] Received unexpected command")
				}
			} else {
				break
			}
		}
	}
}
