use crate::Result;
use std::sync::Arc;
use tokio::runtime::Runtime;

#[tonic::async_trait] // TODO move this somewhere?
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
#[tonic::async_trait]
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
	async fn recv(&self) -> Result<T>;

	/// block until next value is added to the stream without removing any element
	///
	/// this is just an async trait function wrapped by `async_trait`:
	///
	/// `async fn poll(&self) -> codemp::Result<()>;`
	async fn poll(&self) -> Result<()>;

	/// attempt to receive a value without blocking, return None if nothing is available
	fn try_recv(&self) -> Result<Option<T>>;

	/// sync variant of [Self::recv], blocking invoking thread
	fn blocking_recv(&self, rt: &Runtime) -> Result<T> {
		rt.block_on(self.recv())
	}

	/// register a callback to be called for each received stream value
	///
	/// this will spawn a new task on given runtime invoking [Self::recv] in loop and calling given
	/// callback for each received value. a stop channel should be provided, and first value sent
	/// into it will stop the worker loop.
	///
	/// note: creating a callback handler will hold an Arc reference to the given controller,
	/// preventing it from being dropped (and likely disconnecting). using the stop channel is
	/// important for proper cleanup
	fn callback<F>(
		self: &Arc<Self>,
		rt: &tokio::runtime::Runtime,
		mut stop: tokio::sync::mpsc::UnboundedReceiver<()>,
		mut cb: F
	) where
		Self : 'static,
		F : FnMut(T) + Sync + Send + 'static
	{
		let _self = self.clone();
		rt.spawn(async move {
			loop {
				tokio::select! {
					Ok(data) = _self.recv() => cb(data),
					Some(()) = stop.recv() => break,
					else => break,
				}
			}
		});
	}
}
