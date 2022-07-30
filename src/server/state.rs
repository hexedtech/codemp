
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{mpsc, watch};
use tracing::error;

use crate::workspace::Workspace;

#[derive(Debug)]
pub enum AlterState {
	ADD {
		key: String,
		w: Workspace
	},
	REMOVE { key: String },
}

#[derive(Debug)]
pub struct StateManager {
	pub workspaces: watch::Receiver<HashMap<String, Arc<Workspace>>>,
	pub op_tx: mpsc::Sender<AlterState>, // TODO make method for this
	run: watch::Sender<bool>,
}

impl Drop for StateManager {
	fn drop(&mut self) {
		self.run.send(false).unwrap_or_else(|e| {
			error!("Could not stop StateManager worker: {:?}", e);
		})
	}
}

impl StateManager {
	pub fn new() -> Self {
		let (tx, mut rx) = mpsc::channel(32); // TODO quantify backpressure
		let (watch_tx, watch_rx) = watch::channel(HashMap::new());
		let (stop_tx, stop_rx) = watch::channel(true);

		let s = StateManager { 
			workspaces: watch_rx,
			op_tx: tx,
			run: stop_tx,
		};

		tokio::spawn(async move {
			let mut store = HashMap::new();
			while stop_rx.borrow().to_owned() {
				if let Some(event) = rx.recv().await {
					match event {
						AlterState::ADD { key, w } => {
							store.insert(key, Arc::new(w)); // TODO put in hashmap
						},
						AlterState::REMOVE { key } => {
							store.remove(&key);
						},
					}
					watch_tx.send(store.clone()).unwrap();
				} else {
					break
				}
			}
		});

		return s;
	}
}
