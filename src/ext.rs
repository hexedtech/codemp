//! ### Extensions
//! Contains a number of utils used internally or that may be of general interest.

use crate::{api::Controller, errors::ControllerResult};
use tokio::sync::mpsc;

/// Poll all given buffer controllers and wait, returning the first one ready.
///
/// It will spawn tasks blocked on [`Controller::poll`] for each buffer controller.
/// As soon as one finishes, its controller is returned and all other tasks are canceled.
///
/// If a timeout is provided, the result may be `None` if it expires before any task is
/// complete.
///
/// It may return an error if all buffers returned errors while polling.
pub async fn select_buffer(
	buffers: &[crate::buffer::Controller],
	timeout: Option<std::time::Duration>,
	runtime: &tokio::runtime::Runtime
) -> ControllerResult<Option<crate::buffer::Controller>> {
	let (tx, mut rx) = mpsc::unbounded_channel();
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
			},
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
#[derive(Debug)]
pub struct InternallyMutable<T> {
	getter: tokio::sync::watch::Receiver<T>,
	setter: tokio::sync::watch::Sender<T>,
}

impl<T: Default> Default for InternallyMutable<T> {
	fn default() -> Self {
		Self::new(T::default())
	}
}

impl<T> InternallyMutable<T> {
	pub fn new(init: T) -> Self {
		let (tx, rx) = tokio::sync::watch::channel(init);
		Self {
			getter: rx,
			setter: tx,
		}
	}

	pub fn set(&self, state: T) -> T {
		self.setter.send_replace(state)
	}

	pub fn channel(&self) -> tokio::sync::watch::Receiver<T> {
		self.getter.clone()
	}
}

impl<T: Clone> InternallyMutable<T> {
	pub fn get(&self) -> T {
		self.getter.borrow().clone()
	}
}

/// An error that can be ignored with just a warning.
pub trait IgnorableError {
	fn unwrap_or_warn(self, msg: &str);
}

impl<T, E> IgnorableError for std::result::Result<T, E>
where E : std::fmt::Debug {
	/// Logs the error as a warning and returns a unit.
	fn unwrap_or_warn(self, msg: &str) {
		match self {
			Ok(_) => {},
			Err(e) => tracing::warn!("{}: {:?}", msg, e),
		}
	}
}
