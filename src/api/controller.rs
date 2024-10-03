//! # Controller
//!
//! A bidirectional stream handler to easily manage asynchronous operations between local buffers
//! and the server.

use crate::errors::ControllerResult;

// note that we don't use thiserror's #[from] because we don't want the error structs to contain
// these foreign types, and also we want these to be easily constructable

/// Asynchronous and thread-safe handle to a generic bidirectional stream. Exists as a combination
/// of [`AsyncSender`] and [`AsyncReceiver`].
///
/// This generic trait is implemented by actors managing stream procedures, and will generally
/// imply a background worker.
///
/// Events can be enqueued for dispatching without blocking with [`AsyncSender::send`].
///
/// For receiving events from the server, an asynchronous API with [`AsyncReceiver::recv`] is
/// provided; if that is not feasible, consider using [`AsyncReceiver::callback`] or, alternatively,
/// [`AsyncReceiver::poll`] combined with [`AsyncReceiver::try_recv`].
///
/// Every [`Controller`]'s worker will stop cleanly when all references to its [`Controller`] have
/// been dropped.
///
/// [`crate::ext::select_buffer`] may provide a useful helper for managing multiple controllers.
#[allow(async_fn_in_trait)]
#[cfg_attr(feature = "async-trait", async_trait::async_trait)]
pub trait Controller<Tx, Rx = Tx>: AsyncSender<Tx> + AsyncReceiver<Rx>
where
	Tx: Sized + Sync + Send,
	Rx: Sized + Sync + Send,
{
}

/// Asynchronous and thread-safe handle to send data over a stream.
/// See [`Controller`]'s documentation for details.
///
/// Details about the receiving end are left to the implementor.
pub trait AsyncSender<T: Sized + Send + Sync>: Sized + Send + Sync {
	/// Enqueue a new value to be sent to all other users without blocking
	fn send(&self, x: T) -> ControllerResult<()>;
}

/// Asynchronous and thread-safe handle to receive data from a stream.
/// See [`Controller`]'s documentation for details.
///
/// Details about the sender are left to the implementor.
#[allow(async_fn_in_trait)]
#[cfg_attr(feature = "async-trait", async_trait::async_trait)]
pub trait AsyncReceiver<T: Sized + Send + Sync>: Sized + Send + Sync {
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
