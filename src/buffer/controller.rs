//! ### controller
//! 
//! a controller implementation for buffer actions


use std::sync::Arc;

use tokio::sync::{watch, mpsc, RwLock};
use tonic::async_trait;

use crate::errors::IgnorableError;
use crate::{api::Controller, Error};

use crate::api::TextChange;

/// the buffer controller implementation
///
/// this contains
/// * a watch channel which always contains an updated view of the buffer content
/// * a sink to send buffer operations into
/// * a mutexed broadcast receiver for buffer operations
/// * a channel to stop the associated worker
///
/// for each controller a worker exists, managing outgoing and inbound
/// queues, transforming outbound delayed ops and applying remote changes 
/// to the local buffer
///
/// upon dropping this handle will stop the associated worker
#[derive(Debug, Clone)]
pub struct BufferController {
	content: watch::Receiver<String>,
	seen: Arc<RwLock<String>>,
	operations: mpsc::UnboundedSender<TextChange>,
	_stop: Arc<StopOnDrop>, // just exist
}

impl BufferController {
	pub(crate) fn new(
		content: watch::Receiver<String>,
		operations: mpsc::UnboundedSender<TextChange>,
		stop: mpsc::UnboundedSender<()>,
	) -> Self {
		BufferController {
			content, operations,
			_stop: Arc::new(StopOnDrop(stop)),
			seen: Arc::new(RwLock::new("".into())),
		}
	}
}

#[derive(Debug)]
struct StopOnDrop(mpsc::UnboundedSender<()>);

impl Drop for StopOnDrop {
	fn drop(&mut self) {
		self.0.send(()).unwrap_or_warn("could not send stop message to worker");
	}
}

#[async_trait]
impl Controller<TextChange> for BufferController {
	type Input = TextChange;

	async fn poll(&self) -> Result<(), Error> {
		let mut poller = self.content.clone();
		loop {
			poller.changed().await?;
			let seen = self.seen.read().await.clone();
			if *poller.borrow() != seen {
				break
			}
		}
		Ok(())
	}

	fn try_recv(&self) -> Result<Option<TextChange>, Error> {
		let cur = match self.seen.try_read() {
			Err(e) => {
				tracing::error!("try_recv invoked while being mutated: {}", e);
				return Ok(None);
			},
			Ok(x) => x.clone(),
		};
		if *self.content.borrow() != cur {
			match self.seen.try_write() {
				Err(e) => {
					tracing::error!("try_recv mutating while being mutated: {}", e);
					return Ok(None);
				},
				Ok(mut w) => {
					*w = self.content.borrow().clone();
					// TODO it's not the whole buffer that changed
					return Ok(Some(TextChange {
						span: 0..cur.len(),
						content: self.content.borrow().clone(),
						after: "".to_string(),
					}));
				}

			}
		}
		return Ok(None);
	}

	async fn recv(&self) -> Result<TextChange, Error> {
		self.poll().await?;
		match self.try_recv()? {
			Some(x) => Ok(x),
			None => Err(crate::Error::Filler { message: "wtfff".into() }),
		}
	}

	/// enqueue an opseq for processing
	fn send(&self, op: TextChange) -> Result<(), Error> {
		Ok(self.operations.send(op)?)
	}
}
