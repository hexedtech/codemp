//! # Controller
//! 
//! an bidirectional stream handler to easily manage async operations across local buffers and the
//! server

use crate::errors::ControllerResult;

#[async_trait::async_trait]
pub(crate) trait ControllerWorker<T : Sized + Send + Sync> {
	type Controller : Controller<T>;
	type Tx;
	type Rx;

	fn controller(&self) -> Self::Controller;
	async fn work(self, tx: Self::Tx, rx: Self::Rx);
}

// note that we don't use thiserror's #[from] because we don't want the error structs to contain
// these foreign types, and also we want these to be easily constructable

/// async and threadsafe handle to a generic bidirectional stream
///
/// this generic trait is implemented by actors managing stream procedures.
/// events can be enqueued for dispatching without blocking ([Controller::send]), and an async blocking 
/// api ([Controller::recv]) is provided to wait for server events.
///
/// * if possible, prefer a pure [Controller::recv] consumer, awaiting for events
/// * if async is not feasible a [Controller::poll]/[Controller::try_recv] approach is possible
#[async_trait::async_trait]
pub trait Controller<T : Sized + Send + Sync> : Sized + Send + Sync {
	/// enqueue a new value to be sent to all other users
	///
	/// success or failure of this function does not imply validity of sent operation,
	/// because it's integrated asynchronously on the background worker
	async fn send(&self, x: T) -> ControllerResult<()>;

	/// get next value from other users, blocking until one is available
	///
	/// this is just an async trait function wrapped by `async_trait`:
	///
	/// `async fn recv(&self) -> codemp::ControllerResult<T>;`
	async fn recv(&self) -> ControllerResult<T> {
		loop {
			self.poll().await?;
			if let Some(x) = self.try_recv().await? {
				break Ok(x);
			}
		}
	}

	/// registers a callback to be called on receive.
	///
	/// there can only be one callback at any given time.
	fn callback(&self, cb: impl Into<ControllerCallback<Self>>);

	/// clears the currently registered callback.
	fn clear_callback(&self);

	/// block until next value is available without consuming it
	///
	/// this is just an async trait function wrapped by `async_trait`:
	///
	/// `async fn poll(&self) -> codemp::ControllerResult<()>;`
	async fn poll(&self) -> ControllerResult<()>;

	/// attempt to receive a value without blocking, return None if nothing is available
	async fn try_recv(&self) -> ControllerResult<Option<T>>;

	/// stop underlying worker
	///
	/// note that this will mean no more values can be received nor sent,
	/// but existing controllers will still be accessible until all are dropped
	/// 
	/// returns true if stop signal was sent, false if channel is closed
	///  (likely if worker is already stopped)
	fn stop(&self) -> bool;
}


/// type wrapper for Boxed dyn callback
pub struct ControllerCallback<T>(pub Box<dyn Sync + Send + Fn(T)>);

impl<T> ControllerCallback<T> {
	pub fn call(&self, x: T) {
		self.0(x) // lmao at this syntax
	}
}

impl<T> std::fmt::Debug for ControllerCallback<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "ControllerCallback {{ {:p} }}", self.0)
	}
}

impl<T, X: Sync + Send + Fn(T) + 'static> From<X> for ControllerCallback<T> {
	fn from(value: X) -> Self {
		Self(Box::new(value))
	}
}
