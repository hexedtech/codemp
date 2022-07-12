
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{mpsc, watch};

use crate::workspace::Workspace;

#[derive(Debug)]
pub enum AlterState {
	ADD {
		key: String,
		w: Workspace
	},
	REMOVE { key: String },
}

pub struct StateManager {
	store: HashMap<String, Arc<Workspace>>,
	rx:  mpsc::Receiver<AlterState>,
	tx: watch::Sender<HashMap<String, Arc<Workspace>>>
}

impl StateManager {
	pub fn new(rx: mpsc::Receiver<AlterState>, tx: watch::Sender<HashMap<String, Arc<Workspace>>>) -> StateManager {
		StateManager { 
			store: HashMap::new(),
			rx,
			tx
		}
	}

	pub async fn run(mut self) {
		loop {
			if let Some(event) = self.rx.recv().await {
				match event {
					AlterState::ADD { key, w } => {
						self.store.insert(key, Arc::new(w)); // TODO put in hashmap
					},
					AlterState::REMOVE { key } => {
						self.store.remove(&key);
					},
				}
				self.tx.send(self.store.clone()).unwrap();
			} else {
				break
			}
		}
	}
}


pub fn run_state_manager() -> (mpsc::Sender<AlterState>, watch::Receiver<HashMap<String, Arc<Workspace>>>) {
	let (tx, rx) = mpsc::channel(32); // TODO quantify backpressure
	let (watch_tx, watch_rx) = watch::channel(HashMap::new());
	let state = StateManager::new(rx, watch_tx);

	let _task = tokio::spawn(async move {
		state.run().await;
	});

	return (tx, watch_rx);
}
