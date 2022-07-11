use tonic::{transport::Server, Request, Response, Status};

pub mod proto_core {
	tonic::include_proto!("core");
}

use proto_core::session_server::{Session, SessionServer};
use proto_core::{SessionRequest, SessionResponse};


#[derive(Debug, Default)]
pub struct TestSession {}

#[tonic::async_trait]
impl Session for TestSession {
	async fn create(
		&self,
		request: Request<SessionRequest>,
	) -> Result<Response<SessionResponse>, Status> {
		println!("Got a request: {:?}", request);

		let reply = proto_core::SessionResponse {
			session_id: request.into_inner().session_id,
		};

		Ok(Response::new(reply))
	}
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let addr = "[::1]:50051".parse()?;
	let greeter = TestSession::default();

	Server::builder()
		.add_service(SessionServer::new(greeter))
		.serve(addr)
		.await?;

	Ok(())
}

/*

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let mut client = GreeterClient::connect("http://[::1]:50051").await?;
	
	let request = tonic::Request::new(HelloRequest {
		name: "Tonic".into(),
	});
	
	let response = client.say_hello(request).await?;
	
	println!("RESPONSE={:?}", response);
	
	Ok(())
}

*/
