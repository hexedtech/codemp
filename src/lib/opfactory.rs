use operational_transform::{OperationSeq, OTError};
use similar::TextDiff;
use tokio::sync::{mpsc, watch, oneshot};
use tracing::error;

#[derive(Clone)]
pub struct OperationFactory {
	content: String,
}

impl OperationFactory {
	pub fn new(init: Option<String>) -> Self {
		OperationFactory { content: init.unwrap_or(String::new()) }
	}

	// TODO remove the need for this
	pub fn content(&self) -> String {
		self.content.clone()
	}

	pub fn check(&self, txt: &str) -> bool {
		self.content == txt
	}

	pub fn replace(&mut self, txt: &str) -> Result<OperationSeq, OTError> {
		let mut out = OperationSeq::default();
		if self.content == txt { // TODO throw and error rather than wasting everyone's resources
			out.retain(txt.len() as u64);
			return Ok(out); // nothing to do
		}

		let diff = TextDiff::from_chars(self.content.as_str(), txt);

		for change in diff.iter_all_changes() {
			match change.tag() {
				similar::ChangeTag::Equal => out.retain(1),
				similar::ChangeTag::Delete => out.delete(1),
				similar::ChangeTag::Insert => out.insert(change.value()),
			}
		}

		self.content = out.apply(&self.content)?;
		Ok(out)
	}

	pub fn insert(&mut self, txt: &str, pos: u64) -> Result<OperationSeq, OTError> {
		let mut out = OperationSeq::default();
		let total = self.content.len() as u64;
		out.retain(pos);
		out.insert(txt);
		out.retain(total - pos);
		self.content = out.apply(&self.content)?;
		Ok(out)
	}

	pub fn delete(&mut self, pos: u64, count: u64) -> Result<OperationSeq, OTError> {
		let mut out = OperationSeq::default();
		let len = self.content.len() as u64;
		out.retain(pos - count);
		out.delete(count);
		out.retain(len - pos);
		self.content = out.apply(&self.content)?;
		Ok(out)
	}

	pub fn cancel(&mut self, pos: u64, count: u64) -> Result<OperationSeq, OTError> {
		let mut out = OperationSeq::default();
		let len = self.content.len() as u64;
		out.retain(pos);
		out.delete(count);
		out.retain(len - (pos+count));
		self.content = out.apply(&self.content)?;
		Ok(out)
	}

	pub fn process(&mut self, op: OperationSeq) -> Result<String, OTError> {
		self.content = op.apply(&self.content)?;
		Ok(self.content.clone())
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

impl AsyncFactory {
	pub fn new(init: Option<String>) -> Self {
		let (run_tx, run_rx) = watch::channel(true);
		let (ops_tx, ops_rx) = mpsc::channel(64); // TODO hardcoded size
		let (txt_tx, txt_rx) = watch::channel("".into());

		let worker = AsyncFactoryWorker {
			factory: OperationFactory::new(init),
			ops: ops_rx,
			run: run_rx,
			content: txt_tx,
		};

		tokio::spawn(async move { worker.work().await });

		AsyncFactory { run: run_tx, ops: ops_tx, content: txt_rx }
	}

	pub async fn insert(&self, txt: String, pos: u64) -> Result<OperationSeq, OTError> {
		let (tx, rx) = oneshot::channel();
		self.ops.send(OpMsg::Exec(OpWrapper::Insert(txt, pos), tx)).await.map_err(|_| OTError)?;
		rx.await.map_err(|_| OTError)?
	}

	pub async fn delete(&self, pos: u64, count: u64) -> Result<OperationSeq, OTError> {
		let (tx, rx) = oneshot::channel();
		self.ops.send(OpMsg::Exec(OpWrapper::Delete(pos, count), tx)).await.map_err(|_| OTError)?;
		rx.await.map_err(|_| OTError)?
	}

	pub async fn cancel(&self, pos: u64, count: u64) -> Result<OperationSeq, OTError> {
		let (tx, rx) = oneshot::channel();
		self.ops.send(OpMsg::Exec(OpWrapper::Cancel(pos, count), tx)).await.map_err(|_| OTError)?;
		rx.await.map_err(|_| OTError)?
	}

	pub async fn replace(&self, txt: String) -> Result<OperationSeq, OTError> {
		let (tx, rx) = oneshot::channel();
		self.ops.send(OpMsg::Exec(OpWrapper::Replace(txt), tx)).await.map_err(|_| OTError)?;
		rx.await.map_err(|_| OTError)?
	}

	pub async fn process(&self, opseq: OperationSeq) -> Result<String, OTError> {
		let (tx, rx) = oneshot::channel();
		self.ops.send(OpMsg::Process(opseq, tx)).await.map_err(|_| OTError)?;
		rx.await.map_err(|_| OTError)?
	}
}


#[derive(Debug)]
enum OpMsg {
	Exec(OpWrapper, oneshot::Sender<Result<OperationSeq, OTError>>),
	Process(OperationSeq, oneshot::Sender<Result<String, OTError>>),
}

#[derive(Debug)]
enum OpWrapper {
	Insert(String, u64),
	Delete(u64, u64),
	Cancel(u64, u64),
	Replace(String),
}

struct AsyncFactoryWorker {
	factory: OperationFactory,
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
								OpMsg::Exec(op, tx) => tx.send(self.exec(op)).unwrap_or(()),
								OpMsg::Process(opseq, tx) => tx.send(self.factory.process(opseq)).unwrap_or(()),
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
