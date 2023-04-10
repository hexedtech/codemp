use codemp::proto::{RawOp, OperationRequest};
use tokio::sync::{mpsc, broadcast, watch};
use tracing::error;
use md5::Digest;

use operational_transform::OperationSeq;

pub trait BufferStore<T> {
	fn get(&self, key: &T) -> Option<&BufferHandle>;
	fn put(&mut self, key: T, handle: BufferHandle) -> Option<BufferHandle>;

	fn handle(&mut self, key: T, content: Option<String>) {
		let handle = BufferHandle::new(content);
		self.put(key, handle);
	}
}

#[derive(Clone)]
pub struct BufferHandle {
	pub edit: mpsc::Sender<OperationRequest>,
	events: broadcast::Sender<RawOp>,
	pub digest: watch::Receiver<Digest>,
}

impl BufferHandle {
	fn new(init: Option<String>) -> Self {
		let init_val = init.unwrap_or("".into());
		let (edits_tx, edits_rx) = mpsc::channel(64); // TODO hardcoded size
		let (events_tx, _events_rx) = broadcast::channel(64); // TODO hardcoded size
		let (digest_tx, digest_rx) = watch::channel(md5::compute(&init_val));

		let events_tx_clone = events_tx.clone();

		tokio::spawn(async move {
			let worker = BufferWorker {
				content: init_val,
				edits: edits_rx,
				events: events_tx_clone,
				digest: digest_tx,
			};
			worker.work().await
		});

		BufferHandle {
			edit: edits_tx,
			events: events_tx,
			digest: digest_rx,
		}
	}

	pub fn subscribe(&self) -> broadcast::Receiver<RawOp> {
		self.events.subscribe()

	}
}

struct BufferWorker {
	content: String,
	edits: mpsc::Receiver<OperationRequest>,
	events: broadcast::Sender<RawOp>,
	digest: watch::Sender<Digest>,
}

impl BufferWorker {
	async fn work(mut self) {
		loop {
			match self.edits.recv().await {
				None => break,
				Some(v) => {
					let op : OperationSeq = serde_json::from_str(&v.opseq).unwrap();
					match op.apply(&self.content) {
						Ok(res) => {
							self.content = res;
							self.digest.send(md5::compute(&self.content)).unwrap();
							let msg = RawOp {
								opseq: v.opseq,
								user: v.user
							};
							if let Err(e) = self.events.send(msg) {
								error!("could not broadcast OpSeq: {}", e);
							}
						},
						Err(e) => error!("coult not apply OpSeq '{:?}' on '{}' : {}", v, self.content, e),
					}
				},
			}
		}
	}
}
