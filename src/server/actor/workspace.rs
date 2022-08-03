use std::collections::HashMap;

use operational_transform::OperationSeq;
use tokio::sync::{broadcast, mpsc, watch::{self, Ref}};
use tracing::warn;

use super::{buffer::{BufferView, Buffer}, state::User};

type Event = (String, OperationSeq); // TODO jank!

pub enum UserAction {
	ADD {},
	REMOVE {},
}

pub struct WorkspaceView {
	pub rx: broadcast::Receiver<Event>,
	pub tx: mpsc::Sender<BufferAction>,
	pub users: watch::Receiver<HashMap<String, User>>,
	pub buffers: watch::Receiver<HashMap<String, BufferView>>,
}

// Must be clonable, containing references to the actual state maybe? Or maybe give everyone an Arc, idk
#[derive(Debug)]
pub struct Workspace {
	pub name: String,

	buffers: watch::Receiver<HashMap<String, BufferView>>,
	users: watch::Receiver<HashMap<String, User>>,

	pub bus: broadcast::Sender<Event>,

	buf_tx: mpsc::Sender<BufferAction>,
	pub usr_tx: mpsc::Sender<UserAction>,

	run: watch::Sender<bool>,
}

impl Drop for Workspace {
	fn drop(&mut self) {
		self.run.send(false).unwrap_or_else(|e| warn!("could not stop workspace worker: {:?}", e));
	}
}

impl Workspace {
	pub fn new(name: String) -> Self {
		let (buf_tx, mut buf_rx) = mpsc::channel(32);
		let (usr_tx, mut _usr_rx) = mpsc::channel(32);
		let (stop_tx, stop_rx) = watch::channel(true);
		let (buffer_tx, buffer_rx) = watch::channel(HashMap::new());
		let (broadcast_tx, _broadcast_rx) = broadcast::channel(32);
		let (_users_tx, users_rx) = watch::channel(HashMap::new());

		let w = Workspace {
			name,
			run: stop_tx,
			buf_tx,
			usr_tx,
			buffers: buffer_rx,
			bus: broadcast_tx,
			users: users_rx,
		};

		tokio::spawn(async move {
			let mut buffers = HashMap::new();
			while stop_rx.borrow().to_owned() {
				// TODO handle these errors!!
				let action = buf_rx.recv().await.unwrap();
				match action {
					BufferAction::ADD { buffer } => {
						buffers.insert(buffer.view().name.clone(), buffer);
					}
					BufferAction::REMOVE { name } => {
						buffers.remove(&name);
					}
				}
				buffer_tx.send(
					buffers.iter()
						.map(|(k, v)| (k.clone(), v.view()))
						.collect()
				).unwrap();
			}
		});

		return w;
	}

	pub fn buffers_ref(&self) -> Ref<HashMap<String, BufferView>> {
		self.buffers.borrow()
	}

	pub fn users_ref(&self) -> Ref<HashMap<String, User>> {
		self.users.borrow()
	}

	pub fn view(&self) -> WorkspaceView {
		WorkspaceView {
			rx: self.bus.subscribe(),
			tx: self.buf_tx.clone(),
			users: self.users.clone(),
			buffers: self.buffers.clone(),
		}
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

