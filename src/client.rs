use tonic::{transport::Server, Request, Response, Status};

pub mod proto_core {
	tonic::include_proto!("core");
}

use proto_core::session_server::{Session, SessionServer};
use proto_core::{SessionRequest, SessionResponse};

pub fn main() {


}
