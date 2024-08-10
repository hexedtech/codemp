use crate::{Error, api::Controller};
use tokio::sync::mpsc;

/// invoke .poll() on all given buffer controllers and wait, returning the first one ready
///
/// this will spawn tasks blocked on .poll() for each buffer controller. as soon as
/// one finishes, all other tasks will be canceled and the ready controller will be
/// returned
///
/// if timeout is None, result will never be None, otherwise returns None if no buffer
/// is ready before timeout expires
///
/// returns an error if all buffers returned errors while polling.
pub async fn select_buffer(
	buffers: &[crate::buffer::Controller],
	timeout: Option<std::time::Duration>,
	runtime: &tokio::runtime::Runtime
) -> crate::Result<Option<crate::buffer::Controller>> {
	let (tx, mut rx) = mpsc::unbounded_channel();
	let mut tasks = Vec::new();
	for buffer in buffers {
		let _tx = tx.clone();
		let _buffer = buffer.clone();
		tasks.push(runtime.spawn(async move {
			match _buffer.poll().await {
				Ok(()) => _tx.send(Ok(Some(_buffer))),
				Err(_) => _tx.send(Err(Error::Channel { send: true })),
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
			None => return Err(Error::Channel { send: false }),
			Some(Err(_)) => continue, // TODO log errors maybe?
			Some(Ok(x)) => {
				for t in tasks {
					t.abort();
				}
				return Ok(x.clone());
			},
		}
	}
}

/// wraps sender and receiver to allow mutable field with immutable ref
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
		InternallyMutable {
			getter: rx,
			setter: tx,
		}
	}

	pub fn set(&self, state: T) -> T {
		self.setter.send_replace(state)
	}
}

impl<T: Clone> InternallyMutable<T> {
	pub fn get(&self) -> T {
		self.getter.borrow().clone()
	}
}

pub(crate) struct CallbackHandleWatch<T>(pub(crate) tokio::sync::watch::Sender<Option<T>>);

impl<T> crate::api::controller::CallbackHandle for CallbackHandleWatch<T> {
	fn unregister(self) {
		self.0.send_replace(None);
	}
}
