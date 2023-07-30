pub mod workspace;
pub mod cursor;
pub mod errors;
pub mod buffer;

pub use tonic;
pub use tokio;
pub use operational_transform as ot;

use tonic::async_trait;

#[async_trait] // TODO move this somewhere?
pub trait ControllerWorker<T> {
	fn subscribe(&self) -> T;
	async fn work(self);
}

pub mod proto {
	tonic::include_proto!("buffer");
}
