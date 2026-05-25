//! ### Extensions
//! Contains a number of utils used internally or that may be of general interest.


/// Poll all given buffer controllers and wait, returning the first one ready.
///
/// It will spawn tasks blocked on [`AsyncReceiver::poll`] for each buffer controller.
/// As soon as one finishes, its controller is returned and all other tasks are canceled.
///
/// If a timeout is provided, the result may be `None` if it expires before any task is
/// complete.
///
/// It may return an error if all buffers returned errors while polling.
#[cfg(feature = "client")]
pub async fn select_buffer<T: crate::api::CRDT + 'static>(
	buffers: &[crate::client::buffer::Controller<T>],
	timeout: Option<std::time::Duration>,
	runtime: &tokio::runtime::Runtime,
) -> crate::errors::ControllerResult<Option<crate::client::buffer::Controller<T>>> {
	use crate::api::controller::AsyncReceiver;
	let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
	let mut tasks = Vec::new();
	for buffer in buffers {
		let _tx = tx.clone();
		let _buffer = buffer.clone();
		tasks.push(runtime.spawn(async move {
			match _buffer.poll().await {
				Ok(()) => _tx.send(Ok(Some(_buffer))),
				Err(e) => _tx.send(Err(e)),
			}
		}))
	}
	if let Some(d) = timeout {
		let _tx = tx.clone();
		tasks.push(runtime.spawn(async move {
			tokio::time::sleep(d).await;
			_tx.send(Ok(None))
		}));
	}
	loop {
		match rx.recv().await {
			None => return Err(crate::errors::ControllerError::Unfulfilled),
			Some(Err(_)) => continue, // TODO log errors maybe?
			Some(Ok(x)) => {
				for t in tasks {
					t.abort();
				}
				return Ok(x);
			}
		}
	}
}

/// Hash a given byte array with the internally used algorithm.
///
/// Currently, it uses [`xxhash_rust::xxh3::xxh3_64`].
pub fn hash(data: impl AsRef<[u8]>) -> i64 {
	let hash = xxhash_rust::xxh3::xxh3_64(data.as_ref());
	i64::from_ne_bytes(hash.to_ne_bytes())
}

/// A field that can be *internally mutated* regardless of its external mutability.
///
/// Currently, it wraps the [`tokio::sync::watch`] channel couple to achieve this.
#[cfg(feature = "client")]
#[derive(Debug)]
pub struct InternallyMutable<T> {
	getter: tokio::sync::watch::Receiver<T>,
	setter: tokio::sync::watch::Sender<T>,
}

#[cfg(feature = "client")]
impl<T: Default> Default for InternallyMutable<T> {
	fn default() -> Self {
		Self::new(T::default())
	}
}

#[cfg(feature = "client")]
impl<T> InternallyMutable<T> {
	/// Creates a new internally mutable type with the given value.
	pub fn new(init: T) -> Self {
		let (tx, rx) = tokio::sync::watch::channel(init);
		Self {
			getter: rx,
			setter: tx,
		}
	}

	/// Updates the internal value.
	pub fn set(&self, state: T) -> T {
		self.setter.send_replace(state)
	}

	/// Gets the [tokio::sync::watch::Receiver] that can get the internal value.
	pub fn channel(&self) -> tokio::sync::watch::Receiver<T> {
		self.getter.clone()
	}
}

#[cfg(feature = "client")]
impl<T: Clone> InternallyMutable<T> {
	/// Gets and clones the internal value.
	pub fn get(&self) -> T {
		self.getter.borrow().clone()
	}
}

/// An error that can be ignored with just a warning.
pub trait IgnorableError {
	/// Unwraps the error and prints a warning with the contents.
	fn unwrap_or_warn(self, msg: &str);
}

impl<T, E> IgnorableError for std::result::Result<T, E>
where
	E: std::fmt::Debug,
{
	/// Logs the error as a warning and returns a unit.
	fn unwrap_or_warn(self, msg: &str) {
		match self {
			Ok(_) => {}
			Err(e) => tracing::warn!("{}: {:?}", msg, e),
		}
	}
}

#[cfg(feature = "client")]
pub(crate) fn token_to_metadata(
	tok: codemp_proto::common::Token,
) -> tonic::Result<tonic::metadata::MetadataValue<tonic::metadata::Ascii>> {
	tonic::metadata::MetadataValue::try_from(tok.token)
		.map_err(|e| tonic::Status::internal(format!("failed representing token to string: {e}")))
}
