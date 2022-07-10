pub mod proto_core {
	tonic::include_proto!("core");
}

use tonic::transport::Channel;

use proto_core::session_client::SessionClient;
use proto_core::SessionRequest;

use tokio::sync::mpsc;

#[derive(Debug)]
pub struct ConnectionManager {
	client: SessionClient<Channel>,
	rx: mpsc::Receiver<i32>
}


impl ConnectionManager {
	pub async fn new(addr:String, outbound:mpsc::Receiver<i32>) -> Result<Self, Box<dyn std::error::Error>> {
		Ok(ConnectionManager {
			client: SessionClient::connect(addr).await?,
			rx: outbound
		})
	}

	pub async fn process_packets(&mut self) {
		loop {
			if let Some(i) = self.rx.recv().await {
				let request = tonic::Request::new(SessionRequest {session_id: i});
				let response = self.client.create(request).await.unwrap();
				println!("RESPONSE={:?}", response);
			} else {
				break
			}
		}
	}
}
