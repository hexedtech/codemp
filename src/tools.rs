use crate::{Error, api::Controller};

/// invoke .poll() on all buffer controllers and wait, return name of first one ready
///
/// this will spawn tasks for each buffer controller, each blocked in a poll() call. as soon as
/// one finishes, all other tasks will be canceled and the name of ready controller will be
/// returned. just do client.get_buffer(name).try_recv()
///
/// this is not super efficient as of now but has room for improvement. using this API may
/// provide significant improvements on editor-side
pub async fn select_buffer(buffers: &[std::sync::Arc<crate::buffer::Controller>]) -> crate::Result<String> {
	let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
	let mut tasks = Vec::new();
	for buffer in buffers {
		let _tx = tx.clone();
		let _buffer = buffer.clone();
		tasks.push(tokio::spawn(async move {
			match _buffer.poll().await {
				Ok(()) => _tx.send(Ok(_buffer.name.clone())),
				Err(_) => _tx.send(Err(Error::Channel { send: true })),
			}
		}))
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
