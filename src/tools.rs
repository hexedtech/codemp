use crate::{Error, api::Controller};
use std::sync::Arc;
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
	buffers: &[Arc<crate::buffer::Controller>],
	timeout: Option<std::time::Duration>,
) -> crate::Result<Option<Arc<crate::buffer::Controller>>> {
	let (tx, mut rx) = mpsc::unbounded_channel();
	let mut tasks = Vec::new();
	for buffer in buffers {
		let _tx = tx.clone();
		let _buffer = buffer.clone();
		tasks.push(tokio::spawn(async move {
			match _buffer.poll().await {
				Ok(()) => _tx.send(Ok(Some(_buffer))),
				Err(_) => _tx.send(Err(Error::Channel { send: true })),
			}
		}))
	}
	if let Some(d) = timeout {
		let _tx = tx.clone();
		tasks.push(tokio::spawn(async move {
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