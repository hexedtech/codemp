use operational_transform::OperationSeq;
use tokio::sync::{broadcast, mpsc, watch};
use tracing::error;

use library::events::Event;

#[derive(Debug, Clone)]
/// A view of a buffer, with references to access value and send operations
pub struct BufferView {
	pub name: String,
	pub content: watch::Receiver<String>,
	op_tx: mpsc::Sender<OperationSeq>,
}

impl BufferView {
	pub async fn op(&self, op: OperationSeq) -> Result<(), mpsc::error::SendError<OperationSeq>> {
		self.op_tx.send(op).await
	}
}

#[derive(Debug)]
pub struct Buffer {
	view: BufferView,
	run: watch::Sender<bool>,
}

impl Drop for Buffer {
	fn drop(&mut self) {
		self.run.send(false).unwrap_or_else(|e| {
			error!("Could not stop Buffer worker task: {:?}", e);
		});
	}
}

impl Buffer {
	pub fn new(name: String, bus: broadcast::Sender<Event>) -> Self {
		let (op_tx, mut op_rx) = mpsc::channel(32);
		let (stop_tx, stop_rx) = watch::channel(true);
		let (content_tx, content_rx) = watch::channel(String::new());

		let b = Buffer {
			run: stop_tx,
			view: BufferView {
				name: name.clone(),
				op_tx,
				content: content_rx,
			},
		};

		tokio::spawn(async move {
			let mut content = String::new();
			while stop_rx.borrow().to_owned() {
				// TODO handle these errors!!
				let op = op_rx.recv().await.unwrap();
				content = op.apply(content.as_str()).unwrap();
				// bus.send((name.clone(), op)).unwrap(); // TODO fails when there are no receivers subscribed
				content_tx.send(content.clone()).unwrap();
			}
		});

		return b;
	}

	pub fn view(&self) -> BufferView {
		return self.view.clone();
	}
}
