pub mod cursor;
pub mod errors;
pub mod buffer;

pub mod client;

#[cfg(feature = "static")]
pub mod instance;

pub mod prelude;

pub use tonic;
pub use tokio;
pub use operational_transform as ot;

#[cfg(feature = "proto")]
#[allow(non_snake_case)]
pub mod proto {
	tonic::include_proto!("codemp.buffer");
	tonic::include_proto!("codemp.cursor");
}

pub use errors::Error;

#[tonic::async_trait] // TODO move this somewhere?
pub(crate) trait ControllerWorker<T> {
	type Controller : Controller<T>;
	type Tx;
	type Rx;

	fn subscribe(&self) -> Self::Controller;
	async fn work(self, tx: Self::Tx, rx: Self::Rx);
}

#[tonic::async_trait]
pub trait Controller<T> {
	type Input;

	async fn send(&self, x: Self::Input) -> Result<(), Error>;
	async fn recv(&self) -> Result<T, Error>;
}
