use std::{sync::Mutex, collections::VecDeque, ops::Range};

use operational_transform::{OperationSeq, OTError};
use tokio::sync::watch;
use tracing::error;

use super::{OperationFactory, OperationProcessor, op_effective_range};
use crate::errors::IgnorableError;


pub struct OperationController {
	text: Mutex<String>,
	queue: Mutex<VecDeque<OperationSeq>>,
	last: Mutex<watch::Receiver<OperationSeq>>,
	notifier: watch::Sender<OperationSeq>,
	changed: Mutex<watch::Receiver<Range<u64>>>,
	changed_notifier: watch::Sender<Range<u64>>,
	run: watch::Receiver<bool>,
	stop: watch::Sender<bool>,
}

impl OperationController {
	pub fn new(content: String) -> Self {
		let (tx, rx) = watch::channel(OperationSeq::default());
		let (done, wait) = watch::channel(0..0);
		let (stop, run) = watch::channel(true);
		OperationController {
			text: Mutex::new(content),
			queue: Mutex::new(VecDeque::new()),
			last: Mutex::new(rx),
			notifier: tx,
			changed: Mutex::new(wait),
			changed_notifier: done,
			run, stop,
		}
	}

	pub async fn wait(&self) -> Range<u64> {
		let mut blocker = self.changed.lock().unwrap().clone();
		// TODO less jank way
		blocker.changed().await.unwrap_or_log("waiting for changed content #1");
		blocker.changed().await.unwrap_or_log("waiting for changed content #2");
		let span = blocker.borrow().clone();
		span
	}

	pub async fn poll(&self) -> Option<OperationSeq> {
		let len = self.queue.lock().unwrap().len();
		if len <= 0 {
			let mut recv = self.last.lock().unwrap().clone();
			// TODO less jank way
			recv.changed().await.unwrap_or_log("wairing for op changes #1"); // acknowledge current state
			recv.changed().await.unwrap_or_log("wairing for op changes #2"); // wait for a change in state
		}
		Some(self.queue.lock().unwrap().get(0)?.clone())
	}

	pub async fn ack(&self) -> Option<OperationSeq> {
		self.queue.lock().unwrap().pop_front()
	}

	pub fn stop(&self) -> bool {
		match self.stop.send(false) {
			Ok(()) => {
				self.changed_notifier.send(0..0).unwrap_or_log("unlocking downstream for stop");
				self.notifier.send(OperationSeq::default()).unwrap_or_log("unlocking upstream for stop");
				true
			},
			Err(e) => {
				error!("could not send stop signal to workers: {}", e);
				false
			}
		}
	}

	pub fn run(&self) -> bool {
		*self.run.borrow()
	}

	async fn operation(&self, op: &OperationSeq) -> Result<Range<u64>, OTError> {
		let txt = self.content();
		let res = op.apply(&txt)?;
		*self.text.lock().unwrap() = res.clone();
		Ok(op_effective_range(op))
	}
}

impl OperationFactory for OperationController {
	fn content(&self) -> String {
		self.text.lock().unwrap().clone()
	}
}

#[tonic::async_trait]
impl OperationProcessor for OperationController {
	async fn apply(&self, op: OperationSeq) -> Result<Range<u64>, OTError> {
		let span = self.operation(&op).await?;
		self.queue.lock().unwrap().push_back(op.clone());
		self.notifier.send(op.clone()).unwrap_or_log("notifying of applied change");
		Ok(span)
	}


	async fn process(&self, mut op: OperationSeq) -> Result<Range<u64>, OTError> {
		{
			let mut queue = self.queue.lock().unwrap();
			for el in queue.iter_mut() {
				(op, *el) = op.transform(el)?;
			}
		}
		let span = self.operation(&op).await?;
		self.changed_notifier.send(span.clone()).unwrap_or_log("notifying of changed content");
		Ok(span)
	}
}
