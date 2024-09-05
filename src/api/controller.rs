//! # Controller
//! 
//! A bidirectional stream handler to easily manage asynchronous operations between local buffers
//! and the server.

use crate::errors::ControllerResult;

pub(crate) trait ControllerWorker<T : Sized + Send + Sync> {
	type Controller : Controller<T>;
	type Tx;
	type Rx;

	fn controller(&self) -> Self::Controller;
	async fn work(self, tx: Self::Tx, rx: Self::Rx);
}

// note that we don't use thiserror's #[from] because we don't want the error structs to contain
// these foreign types, and also we want these to be easily constructable

/// Asynchronous and thread-safe handle to a generic bidirectional stream.
///
/// This generic trait is implemented by actors managing stream procedures.
/// 
/// Events can be enqueued for dispatching without blocking with [`Controller::send`].
///
/// For receiving events from the server, an asynchronous API with [`Controller::recv`] is
/// provided; if that is not feasible, consider using [`Controller::callback`] or, alternatively,
/// [`Controller::poll`] combined with [`Controller::try_recv`].
///
/// [`crate::ext::select_buffer`] may provide a useful helper for managing multiple controllers.
#[allow(async_fn_in_trait)]
#[cfg_attr(feature = "async-trait", async_trait::async_trait)]
pub trait Controller<T : Sized + Send + Sync> : Sized + Send + Sync {
	/// Enqueue a new value to be sent to all other users.
	async fn send(&self, x: T) -> ControllerResult<()>;

	/// Block until a value is available and returns it.
	async fn recv(&self) -> ControllerResult<T> {
		loop {
			self.poll().await?;
			if let Some(x) = self.try_recv().await? {
				break Ok(x);
			}
		}
	}

	/// Register a callback to be called on receive.
	///
	/// There can only be one callback registered at any given time.
	fn callback(&self, cb: impl Into<ControllerCallback<Self>>);

	/// Clear the currently registered callback.
	fn clear_callback(&self);

	/// Block until a value is available, without consuming it.
	async fn poll(&self) -> ControllerResult<()>;

	/// Attempt to receive a value, return None if nothing is currently available.
	async fn try_recv(&self) -> ControllerResult<Option<T>>;

	/// Stop underlying worker.
	///
	/// After this is called, nothing can be received or sent anymore; however, existing
	/// controllers will still be accessible until all handles are dropped.
	/// 
	/// Returns true if the stop signal was successfully sent, false if channel was
	/// closed (probably because worker had already been stopped).
	fn stop(&self) -> bool;
}


/// Type wrapper for Boxed dynamic callback.
pub struct ControllerCallback<T>(pub Box<dyn Sync + Send + Fn(T)>);

impl<T> ControllerCallback<T> {
	pub(crate) fn call(&self, x: T) {
		self.0(x)
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
