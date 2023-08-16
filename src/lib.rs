pub mod cursor;
pub mod errors;
pub mod buffer;

pub mod state;

pub use tonic;
pub use tokio;
pub use operational_transform as ot;

#[cfg(feature = "proto")]
#[allow(non_snake_case)]
pub mod proto {
	tonic::include_proto!("codemp.buffer");
	tonic::include_proto!("codemp.cursor");
}

pub use errors::CodempError;

#[tonic::async_trait] // TODO move this somewhere?
pub(crate) trait ControllerWorker<T> {
	fn subscribe(&self) -> T;
	async fn work(self);
}

#[tonic::async_trait]
pub trait Controller<T> {
	async fn recv(&self) -> Result<T, CodempError>;
	async fn send(&self, x: T) -> Result<(), CodempError>;
}
