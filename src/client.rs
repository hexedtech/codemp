pub mod proto_core {
	tonic::include_proto!("core");
}

use proto_core::session_client::SessionClient;
use proto_core::SessionRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let mut client = SessionClient::connect("http://[::1]:50051").await?;

	let request = tonic::Request::new(SessionRequest {
		session_id: 0,
	});

	let response = client.create(request).await?;

	println!("RESPONSE={:?}", response);

	Ok(())
}
