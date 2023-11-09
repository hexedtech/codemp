//! # Controller
//! 
//! an bidirectional stream handler to easily manage async operations across local buffers and the
//! server

use crate::Result;

#[async_trait::async_trait]
pub(crate) trait ControllerWorker<T : Sized + Send + Sync> {
	type Controller : Controller<T>;
	type Tx;
	type Rx;

	fn subscribe(&self) -> Self::Controller;
	async fn work(self, tx: Self::Tx, rx: Self::Rx);
}

/// async and threadsafe handle to a generic bidirectional stream
///
/// this generic trait is implemented by actors managing stream procedures.
/// events can be enqueued for dispatching without blocking ([Controller::send]), and an async blocking 
/// api ([Controller::recv]) is provided to wait for server events. Additional sync blocking
/// ([Controller::blocking_recv]) and callback-based ([Controller::callback]) are implemented.
///
/// * if possible, prefer a pure [Controller::recv] consumer
/// * a second possibility in preference is using a [Controller::callback]
/// * if neither is feasible a [Controller::poll]/[Controller::try_recv] approach is available
#[async_trait::async_trait]
pub trait Controller<T : Sized + Send + Sync> : Sized + Send + Sync {
	/// type of upstream values, used in [Self::send]
	type Input;

	/// enqueue a new value to be sent
	fn send(&self, x: Self::Input) -> Result<()>;

	/// get next value from stream, blocking until one is available
	///
	/// this is just an async trait function wrapped by `async_trait`:
	///
	/// `async fn recv(&self) -> codemp::Result<T>;`
	async fn recv(&self) -> Result<T> {
		if let Some(x) = self.try_recv()? {
			return Ok(x); // short circuit if already available
		}

		self.poll().await?;
		Ok(self.try_recv()?.expect("no message available after polling"))
	}

	/// block until next value is added to the stream without removing any element
	///
	/// this is just an async trait function wrapped by `async_trait`:
	///
	/// `async fn poll(&self) -> codemp::Result<()>;`
	async fn poll(&self) -> Result<()>;

	/// attempt to receive a value without blocking, return None if nothing is available
	fn try_recv(&self) -> Result<Option<T>>;

	/// sync variant of [Self::recv], blocking invoking thread
	fn blocking_recv(&self, rt: &tokio::runtime::Handle) -> Result<T> {
		rt.block_on(self.recv())
	}
}
