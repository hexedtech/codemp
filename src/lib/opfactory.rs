use std::sync::Arc;

use operational_transform::{OperationSeq, OTError};
use similar::TextDiff;
use tokio::sync::{mpsc, watch, oneshot};
use tracing::{error, warn};

#[tonic::async_trait]
pub trait OperationFactory {
	fn content(&self) -> String;
	async fn apply(&self, op: OperationSeq) -> Result<String, OTError>;
	async fn process(&self, op: OperationSeq) -> Result<String, OTError>;
	async fn acknowledge(&self, op: OperationSeq) -> Result<(), OTError>;

	fn replace(&self, txt: &str) -> OperationSeq {
		let mut out = OperationSeq::default();
		if self.content() == txt {
			return out; // TODO this won't work, should we return a noop instead?
		}

		let diff = TextDiff::from_chars(self.content().as_str(), txt);

		for change in diff.iter_all_changes() {
			match change.tag() {
				similar::ChangeTag::Equal => out.retain(1),
				similar::ChangeTag::Delete => out.delete(1),
				similar::ChangeTag::Insert => out.insert(change.value()),
			}
		}

		out
	}

	fn insert(&self, txt: &str, pos: u64) -> OperationSeq {
		let mut out = OperationSeq::default();
		let total = self.content().len() as u64;
		out.retain(pos);
		out.insert(txt);
		out.retain(total - pos);
		out
	}

	fn delete(&self, pos: u64, count: u64) -> OperationSeq {
		let mut out = OperationSeq::default();
		let len = self.content().len() as u64;
		out.retain(pos - count);
		out.delete(count);
		out.retain(len - pos);
		out
	}

	fn cancel(&self, pos: u64, count: u64) -> OperationSeq {
		let mut out = OperationSeq::default();
		let len = self.content().len() as u64;
		out.retain(pos);
		out.delete(count);
		out.retain(len - (pos+count));
		out
	}
}

pub struct AsyncFactory {
	run: watch::Sender<bool>,
	ops: mpsc::Sender<OpMsg>,
	#[allow(unused)] // TODO is this necessary?
	content: watch::Receiver<String>,
}

impl Drop for AsyncFactory {
	fn drop(&mut self) {
		self.run.send(false).unwrap_or(());
	}
}

#[tonic::async_trait]
impl OperationFactory for AsyncFactory {
	fn content(&self) -> String {
		return self.content.borrow().clone();
	}

	async fn apply(&self, op: OperationSeq) -> Result<String, OTError> {
		let (tx, rx) = oneshot::channel();
		self.ops.send(OpMsg::Apply(op, tx)).await.map_err(|_| OTError)?;
		Ok(rx.await.map_err(|_| OTError)?)
	}

	async fn process(&self, op: OperationSeq) -> Result<String, OTError> {
		let (tx, rx) = oneshot::channel();
		self.ops.send(OpMsg::Process(op, tx)).await.map_err(|_| OTError)?;
		Ok(rx.await.map_err(|_| OTError)?)
	}

	async fn acknowledge(&self, op: OperationSeq) -> Result<(), OTError> {
		let (tx, rx) = oneshot::channel();
		self.ops.send(OpMsg::Acknowledge(op, tx)).await.map_err(|_| OTError)?;
		Ok(rx.await.map_err(|_| OTError)?)
	}

}

impl AsyncFactory {
	pub fn new(init: Option<String>) -> Self {
		let (run_tx, run_rx) = watch::channel(true);
		let (ops_tx, ops_rx) = mpsc::channel(64); // TODO hardcoded size
		let (txt_tx, txt_rx) = watch::channel("".into());

		let worker = AsyncFactoryWorker {
			text: init.unwrap_or("".into()),
			ops: ops_rx,
			run: run_rx,
			content: txt_tx,
		};

		tokio::spawn(async move { worker.work().await });

		AsyncFactory { run: run_tx, ops: ops_tx, content: txt_rx }
	}
}


#[derive(Debug)]
enum OpMsg {
	Apply(OperationSeq, oneshot::Sender<String>),
	Process(OperationSeq, oneshot::Sender<String>),
	Acknowledge(OperationSeq, oneshot::Sender<()>)
}

struct AsyncFactoryWorker {
	text: String,
	ops: mpsc::Receiver<OpMsg>,
	run: watch::Receiver<bool>,
	content: watch::Sender<String>
}

impl AsyncFactoryWorker {
	async fn work(mut self) {
		while *self.run.borrow() {
			tokio::select! { // periodically check run so that we stop cleanly

				recv = self.ops.recv() => {
					match recv {
						Some(msg) => {
							match msg {
								OpMsg::Apply(op, tx) => tx.send(self.exec(op)).unwrap_or(()),
								OpMsg::Process(opseq, tx) => tx.send(self.factory.process(opseq)).unwrap_or(()),
								OpMsg::Ack(opseq, tx) => tx.send(self.factory.ack(opseq)).unwrap_or(()),
							}
							if let Err(e) = self.content.send(self.factory.content()) {
								error!("error updating content: {}", e);
								break;
							}
						},
						None => break,
					}
				},

				_ = tokio::time::sleep(std::time::Duration::from_secs(1)) => {},

			};
		}
	}

	fn exec(&mut self, op: OpWrapper) -> Result<OperationSeq, OTError> {
		match op {
			OpWrapper::Insert(txt, pos) => Ok(self.factory.insert(&txt, pos)?),
			OpWrapper::Delete(pos, count) => Ok(self.factory.delete(pos, count)?),
			OpWrapper::Cancel(pos, count) => Ok(self.factory.cancel(pos, count)?),
			OpWrapper::Replace(txt) => Ok(self.factory.replace(&txt)?),
		}
	}
}
