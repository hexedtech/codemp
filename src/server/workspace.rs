use std::collections::HashMap;

use operational_transform::OperationSeq;
use tokio::sync::{broadcast, mpsc, watch};
use tracing::error;

#[derive(Debug, Clone)]
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
	pub fn new(name: String, bus: broadcast::Sender<(String, OperationSeq)>) -> Self {
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
				bus.send((name.clone(), op)).unwrap(); // TODO fails when there are no receivers subscribed
				content_tx.send(content.clone()).unwrap();
			}
		});

		return b;
	}

	pub fn view(&self) -> BufferView {
		return self.view.clone();
	}
}

pub struct WorkspaceView {
	pub rx: broadcast::Receiver<OperationSeq>,
	pub tx: mpsc::Sender<OperationSeq>,
}

// Must be clonable, containing references to the actual state maybe? Or maybe give everyone an Arc, idk
#[derive(Debug)]
pub struct Workspace {
	pub name: String,
	pub buffers: watch::Receiver<HashMap<String, BufferView>>,
	pub bus: broadcast::Sender<(String, OperationSeq)>,
	op_tx: mpsc::Sender<BufferAction>,
	run: watch::Sender<bool>,
}

impl Workspace {
	pub fn new(name: String) -> Self {
		let (op_tx, mut op_rx) = mpsc::channel(32);
		let (stop_tx, stop_rx) = watch::channel(true);
		let (buf_tx, buf_rx) = watch::channel(HashMap::new());
		let (broadcast_tx, broadcast_rx) = broadcast::channel(32);

		let w = Workspace {
			name,
			run: stop_tx,
			op_tx,
			buffers: buf_rx,
			bus: broadcast_tx,
		};

		tokio::spawn(async move {
			let mut buffers = HashMap::new();
			while stop_rx.borrow().to_owned() {
				// TODO handle these errors!!
				let action = op_rx.recv().await.unwrap();
				match action {
					BufferAction::ADD { buffer } => {
						buffers.insert(buffer.view.name.clone(), buffer);
					}
					BufferAction::REMOVE { name } => {
						buffers.remove(&name);
					}
				}
				buf_tx.send(
					buffers.iter()
						.map(|(k, v)| (k.clone(), v.view()))
						.collect()
				).unwrap();
			}
		});

		return w;
	}
}

pub enum BufferAction {
	ADD {
		buffer: Buffer,
	},
	REMOVE {
		name: String, // TODO remove by id?
	},
}

// impl Default for Workspace {
// 	fn default() -> Self {
// 		Workspace {
// 			name: "fuck you".to_string(),
// 			content: "too".to_string(),
// 		}
// 	}
// }
