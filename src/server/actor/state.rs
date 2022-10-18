
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{mpsc, watch};
use tracing::error;
use uuid::Uuid;

use crate::actor::workspace::Workspace;


#[derive(Debug)]
enum WorkspaceAction {
	ADD {
		key: Uuid,
		w: Box<Workspace>,
	},
	REMOVE {
		key: Uuid
	},
}

#[derive(Debug, Clone)]
pub struct WorkspacesView {
	watch: watch::Receiver<HashMap<Uuid, Arc<Workspace>>>,
	op: mpsc::Sender<WorkspaceAction>,
}

impl WorkspacesView {
	pub fn borrow(&self) -> watch::Ref<HashMap<Uuid, Arc<Workspace>>> {
		self.watch.borrow()
	}

	pub async fn add(&mut self, w: Workspace) {
		self.op.send(WorkspaceAction::ADD { key: w.id, w: Box::new(w) }).await.unwrap();
	}

	pub async fn remove(&mut self, key: Uuid) {
		self.op.send(WorkspaceAction::REMOVE { key }).await.unwrap();
	}
}

#[derive(Debug)]
pub struct StateManager {
	pub workspaces: WorkspacesView,
	pub run: watch::Receiver<bool>,
	run_tx: watch::Sender<bool>,
}

impl Drop for StateManager {
	fn drop(&mut self) {
		self.run_tx.send(false).unwrap_or_else(|e| {
			error!("Could not stop StateManager worker: {:?}", e);
		})
	}
}

impl StateManager {
	pub fn new() -> Self {
		let (tx, rx) = mpsc::channel(32); // TODO quantify backpressure
		let (workspaces_tx, workspaces_rx) = watch::channel(HashMap::new());
		let (run_tx, run_rx) = watch::channel(true);

		let s = StateManager { 
			workspaces: WorkspacesView { watch: workspaces_rx, op: tx },
			run_tx, run: run_rx,
		};

		s.workspaces_worker(rx, workspaces_tx);

		return s;
	}

	fn workspaces_worker(&self, mut rx: mpsc::Receiver<WorkspaceAction>, tx: watch::Sender<HashMap<Uuid, Arc<Workspace>>>) {
		let run = self.run.clone();
		tokio::spawn(async move {
			let mut store = HashMap::new();

			while run.borrow().to_owned() {
				if let Some(event) = rx.recv().await {
					match event {
						WorkspaceAction::ADD { key, w } => {
							store.insert(key, Arc::new(*w)); // TODO put in hashmap
						},
						WorkspaceAction::REMOVE { key } => {
							store.remove(&key);
						},
					}
					tx.send(store.clone()).unwrap();
				} else {
					break
				}
			}
		});
	}

	pub fn view(&self) -> WorkspacesView {
		return self.workspaces.clone();
	}

	/// get a workspace Arc directly, without passing by the WorkspacesView
	pub fn get(&self, key: &Uuid) -> Option<Arc<Workspace>> {
		if let Some(w) = self.workspaces.borrow().get(key) {
			return Some(w.clone());
		}
		return None;
	}
}
