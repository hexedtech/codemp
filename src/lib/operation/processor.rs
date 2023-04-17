use std::{sync::{Mutex, Arc}, collections::VecDeque};

use operational_transform::{OperationSeq, OTError};
use tokio::sync::{watch, oneshot, mpsc};
use tracing::error;

use crate::operation::factory::OperationFactory;


#[tonic::async_trait]
pub trait OperationProcessor : OperationFactory{
	async fn apply(&self, op: OperationSeq) -> Result<String, OTError>;
	async fn process(&self, op: OperationSeq) -> Result<String, OTError>;

	async fn poll(&self) -> Option<OperationSeq>;
	async fn ack(&self)  -> Option<OperationSeq>;
}


pub struct OperationController {
	text: Mutex<String>,
	queue: Mutex<VecDeque<OperationSeq>>,
	last: Mutex<watch::Receiver<OperationSeq>>,
	notifier: watch::Sender<OperationSeq>,
}

impl OperationController {
	pub fn new(content: String) -> Self {
		let (tx, rx) = watch::channel(OperationSeq::default());
		OperationController {
			text: Mutex::new(content),
			queue: Mutex::new(VecDeque::new()),
			last: Mutex::new(rx),
			notifier: tx,
		}
	}
}

impl OperationFactory for OperationController {
	fn content(&self) -> String {
		self.text.lock().unwrap().clone()
	}
}

#[tonic::async_trait]
impl OperationProcessor for OperationController {
	async fn apply(&self, op: OperationSeq) -> Result<String, OTError> {
		let txt = self.content();
		let res = op.apply(&txt)?;
		*self.text.lock().unwrap() = res.clone();
		self.queue.lock().unwrap().push_back(op.clone());
		self.notifier.send(op).unwrap();
		Ok(res)
	}

	async fn process(&self, mut op: OperationSeq) -> Result<String, OTError> {
		let mut queue = self.queue.lock().unwrap();
		for el in queue.iter_mut() {
			(op, *el) = op.transform(el)?;
		}
		let txt = self.content();
		let res = op.apply(&txt)?;
		*self.text.lock().unwrap() = res.clone();
		Ok(res)
	}

	async fn poll(&self) -> Option<OperationSeq> {
		let len = self.queue.lock().unwrap().len();
		if len <= 0 {
			let mut recv = self.last.lock().unwrap().clone();
			recv.changed().await.unwrap();
		}
		Some(self.queue.lock().unwrap().get(0)?.clone())
	}

	async fn ack(&self) -> Option<OperationSeq> {
		self.queue.lock().unwrap().pop_front()
	}
}
