use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use similar::{TextDiff, ChangeTag};
use tokio::sync::{watch, mpsc, oneshot};
use tonic::{async_trait, Streaming};
use uuid::Uuid;
use woot::crdt::{Op, CRDT, TextEditor};
use woot::woot::Woot;

use crate::errors::IgnorableError;
use crate::api::controller::ControllerWorker;
use crate::api::TextChange;
use crate::proto::buffer_service::Operation;

use super::controller::BufferController;

pub(crate) struct BufferWorker {
	_user_id: Uuid,
	name: String,
	buffer: Woot,
	content: watch::Sender<String>,
	operations: mpsc::UnboundedReceiver<TextChange>,
	poller: mpsc::UnboundedReceiver<oneshot::Sender<()>>,
	pollers: Vec<oneshot::Sender<()>>,
	handles: ClonableHandlesForController,
	stop: mpsc::UnboundedReceiver<()>,
}

struct ClonableHandlesForController {
	operations: mpsc::UnboundedSender<TextChange>,
	poller: mpsc::UnboundedSender<oneshot::Sender<()>>,
	stop: mpsc::UnboundedSender<()>,
	content: watch::Receiver<String>,
}

impl BufferWorker {
	pub fn new(user_id: Uuid, path: &str) -> Self {
		let (txt_tx, txt_rx) = watch::channel("".to_string());
		let (op_tx, op_rx) = mpsc::unbounded_channel();
		let (end_tx, end_rx) = mpsc::unbounded_channel();
		let (poller_tx, poller_rx) = mpsc::unbounded_channel();
		let mut hasher = DefaultHasher::new();
		user_id.hash(&mut hasher);
		let site_id = hasher.finish() as usize;
		BufferWorker {
			_user_id: user_id,
			name: path.to_string(),
			buffer: Woot::new(site_id % (2<<10), ""), // TODO remove the modulo, only for debugging!
			content: txt_tx,
			operations: op_rx,
			poller: poller_rx,
			pollers: Vec::new(),
			handles: ClonableHandlesForController {
				operations: op_tx,
				poller: poller_tx,
				stop: end_tx,
				content: txt_rx,
			},
			stop: end_rx,
		}
	}
}

#[async_trait]
impl ControllerWorker<TextChange> for BufferWorker {
	type Controller = BufferController;
	type Tx = mpsc::Sender<Operation>;
	type Rx = Streaming<Operation>;

	fn subscribe(&self) -> BufferController {
		BufferController::new(
			self.name.clone(),
			self.handles.content.clone(),
			self.handles.operations.clone(),
			self.handles.poller.clone(),
			self.handles.stop.clone(),
		)
	}

	async fn work(mut self, tx: Self::Tx, mut rx: Self::Rx) {
		loop {
			// block until one of these is ready
			tokio::select! {
				biased;

				// received stop signal
				_ = self.stop.recv() => break,

				// received a new poller, add it to collection
				res = self.poller.recv() => match res {
					None => break tracing::error!("poller channel closed"),
					Some(tx) => self.pollers.push(tx),
				},

				// received a text change from editor
				res = self.operations.recv() => match res {
					None => break,
					Some(change) => {
						if !change.is_empty() {
							let view = self.buffer.view();
							match view.get(change.span.clone()) {
								None =>  tracing::error!("received illegal span from client: {:?} but buffer is of len {}", change.span, view.len()),
								Some(span) => {
									let diff = TextDiff::from_chars(span, &change.content);

									let mut i = 0;
									let mut ops = Vec::new();
									for diff in diff.iter_all_changes() {
										match diff.tag() {
											ChangeTag::Equal => i += 1,
											ChangeTag::Delete => match self.buffer.delete(change.span.start + i) {
												Ok(op) => ops.push(op),
												Err(e) => tracing::error!("could not apply deletion: {}", e),
											},
											ChangeTag::Insert => {
												for c in diff.value().chars() {
													match self.buffer.insert(change.span.start + i, c) {
														Ok(op) => {
															ops.push(op);
															i += 1;
														},
														Err(e) => tracing::error!("could not apply insertion: {}", e),
													}
												}
											},
										}
									}

									for op in ops {
										let operation = Operation { 
											data: postcard::to_extend(&op, Vec::new()).unwrap(),
											user: None,
											path: Some(self.name.clone())
										};
	
										match tx.send(operation).await {
											Err(e) => tracing::error!("server refused to broadcast {}: {}", op, e),
											Ok(()) => {
												self.content.send(self.buffer.view()).unwrap_or_warn("could not send buffer update");
											},
										}
									}
								},
							}
						}
					}
				},

				// received a message from server
				res = rx.message() => match res {
					Err(_e) => break,
					Ok(None) => break,
					Ok(Some(change)) => match postcard::from_bytes::<Op>(&change.data) {
						Ok(op) => {
							self.buffer.merge(op);
							self.content.send(self.buffer.view()).unwrap_or_warn("could not send buffer update");
							for tx in self.pollers.drain(..) {
								tx.send(()).unwrap_or_warn("could not wake up poller");
							}
						},
						Err(e) => tracing::error!("could not deserialize operation from server: {}", e),
					},
				},
			}
		}
	}
}
