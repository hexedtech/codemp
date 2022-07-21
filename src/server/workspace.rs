use std::sync::Arc;

use operational_transform::OperationSeq;
use tokio::sync::{broadcast, mpsc};

pub struct WorkspaceView {
	pub rx: broadcast::Receiver<OperationSeq>,
	pub tx: mpsc::Sender<OperationSeq>,
}

// Must be clonable, containing references to the actual state maybe? Or maybe give everyone an Arc, idk
#[derive(Debug)]
pub struct Workspace {
	pub name: String,
	pub content: String,
	pub tx: mpsc::Sender<OperationSeq>,
	w_tx: Arc<broadcast::Sender<OperationSeq>>,
}

impl Workspace {
	pub fn new(
		name: String,
		content: String,
		tx: mpsc::Sender<OperationSeq>,
		w_tx: Arc<broadcast::Sender<OperationSeq>>,
	) -> Self {
		Workspace {
			name,
			content,
			tx,
			w_tx,
		}
	}

	pub fn view(&self) -> WorkspaceView {
		WorkspaceView {
			rx: self.w_tx.subscribe(),
			tx: self.tx.clone(),
		}
	}
}

pub async fn worker(
	mut w: Workspace,
	tx: Arc<broadcast::Sender<OperationSeq>>,
	mut rx: mpsc::Receiver<OperationSeq>,
) {
	loop {
		if let Some(op) = rx.recv().await {
			w.content = op.apply(&w.content).unwrap();
			tx.send(op).unwrap();
		} else {
			break;
		}
	}
}

// impl Default for Workspace {
// 	fn default() -> Self {
// 		Workspace {
// 			name: "fuck you".to_string(),
// 			content: "too".to_string(),
// 		}
// 	}
// }
