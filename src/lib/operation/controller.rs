use std::{sync::Mutex, collections::VecDeque, ops::Range};

use operational_transform::{OperationSeq, OTError};
use tokio::sync::watch;
use tracing::{warn, error};

use super::{OperationFactory, OperationProcessor, op_effective_range};


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
		ignore_and_log(blocker.changed().await, "waiting for changed content #1");
		ignore_and_log(blocker.changed().await, "waiting for changed content #2");
		let span = blocker.borrow().clone();
		span
	}

	pub async fn poll(&self) -> Option<OperationSeq> {
		let len = self.queue.lock().unwrap().len();
		if len <= 0 {
			let mut recv = self.last.lock().unwrap().clone();
			// TODO less jank way
			ignore_and_log(recv.changed().await, "wairing for op changes #1"); // acknowledge current state
			ignore_and_log(recv.changed().await, "wairing for op changes #2"); // wait for a change in state
		}
		Some(self.queue.lock().unwrap().get(0)?.clone())
	}

	pub async fn ack(&self) -> Option<OperationSeq> {
		self.queue.lock().unwrap().pop_front()
	}

	pub fn stop(&self) -> bool {
		match self.stop.send(false) {
			Ok(()) => true,
			Err(e) => {
				error!("could not send stop signal to workers: {}", e);
				false
			}
		}
	}

	pub fn run(&self) -> bool {
		*self.run.borrow()
	}
}

impl OperationFactory for OperationController {
	fn content(&self) -> String {
		self.text.lock().unwrap().clone()
	}
}

/// TODO properly handle errors rather than sinking them all in here!
fn ignore_and_log<T, E : std::fmt::Display>(x: Result<T, E>, msg: &str) {
	match x {
		Ok(_) => {},
		Err(e) => {
			warn!("ignored error {}: {}", msg, e);
		}
	}
}

#[tonic::async_trait]
impl OperationProcessor for OperationController {
	async fn apply(&self, op: OperationSeq) -> Result<Range<u64>, OTError> {
		let txt = self.content();
		let res = op.apply(&txt)?;
		*self.text.lock().unwrap() = res.clone();
		self.queue.lock().unwrap().push_back(op.clone());
		ignore_and_log(self.notifier.send(op.clone()), "notifying of applied change");
		Ok(op_effective_range(&op))
	}


	async fn process(&self, mut op: OperationSeq) -> Result<Range<u64>, OTError> {
		let mut queue = self.queue.lock().unwrap();
		for el in queue.iter_mut() {
			(op, *el) = op.transform(el)?;
		}
		let txt = self.content();
		let res = op.apply(&txt)?;
		let span = op_effective_range(&op);
		*self.text.lock().unwrap() = res.clone();
		ignore_and_log(self.changed_notifier.send(span.clone()), "notifying of changed content");
		Ok(span)
	}
}
