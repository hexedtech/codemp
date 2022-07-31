use std::collections::HashMap;

use operational_transform::OperationSeq;
use tokio::sync::{broadcast, mpsc, watch};

use super::buffer::{BufferView, Buffer};

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
						buffers.insert(buffer.view().name.clone(), buffer);
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

