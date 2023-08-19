pub mod cursor;
pub mod buffer;
pub mod errors;
pub mod client;
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

use std::sync::Arc;

#[tonic::async_trait] // TODO move this somewhere?
pub(crate) trait ControllerWorker<T> {
	type Controller : Controller<T>;
	type Tx;
	type Rx;

	fn subscribe(&self) -> Self::Controller;
	async fn work(self, tx: Self::Tx, rx: Self::Rx);
}

#[tonic::async_trait]
pub trait Controller<T> : Sized + Send + Sync {
	type Input;

	async fn send(&self, x: Self::Input) -> Result<(), Error>;
	async fn recv(&self) -> Result<T, Error>;

	fn callback<F>(
		self: Arc<Self>,
		rt: &tokio::runtime::Runtime,
		mut stop: tokio::sync::mpsc::UnboundedReceiver<()>,
		mut cb: F
	) where
		Self : 'static,
		F : FnMut(T) + Sync + Send + 'static
	{
		rt.spawn(async move {
			loop {
				tokio::select! {
					Ok(data) = self.recv() => cb(data),
					Some(()) = stop.recv() => break,
					else => break,
				}
			}
		});
	}
}
