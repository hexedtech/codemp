use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use similar::{TextDiff, ChangeTag};
use tokio::sync::{watch, mpsc};
use tonic::transport::Channel;
use tonic::{async_trait, Streaming};
use woot::crdt::{Op, CRDT, TextEditor};
use woot::woot::Woot;

use crate::errors::IgnorableError;
use crate::proto::{OperationRequest, RawOp};
use crate::proto::buffer_client::BufferClient;
use crate::api::controller::ControllerWorker;

use super::TextChange;
use super::controller::BufferController;


pub(crate) struct BufferControllerWorker {
	uid: String,
	content: watch::Sender<String>,
	operations: mpsc::UnboundedReceiver<TextChange>,
	receiver: watch::Receiver<String>,
	sender: mpsc::UnboundedSender<TextChange>,
	buffer: Woot,
	path: String,
	stop: mpsc::UnboundedReceiver<()>,
	stop_control: mpsc::UnboundedSender<()>,
}

impl BufferControllerWorker {
	pub fn new(uid: String, path: &str) -> Self {
		let (txt_tx, txt_rx) = watch::channel("".to_string());
		let (op_tx, op_rx) = mpsc::unbounded_channel();
		let (end_tx, end_rx) = mpsc::unbounded_channel();
		let mut hasher = DefaultHasher::new();
		uid.hash(&mut hasher);
		let site_id = hasher.finish() as usize;
		BufferControllerWorker {
			uid,
			content: txt_tx,
			operations: op_rx,
			receiver: txt_rx,
			sender: op_tx,
			buffer: Woot::new(site_id, ""), // TODO initialize with buffer!
			path: path.to_string(),
			stop: end_rx,
			stop_control: end_tx,
		}
	}

	async fn send_op(&self, tx: &mut BufferClient<Channel>, outbound: &Op) -> crate::Result<()> {
		let opseq = serde_json::to_string(outbound).expect("could not serialize opseq");
		let req = OperationRequest {
			path: self.path.clone(),
			hash: format!("{:x}", md5::compute(self.buffer.view())),
			op: Some(RawOp {
				opseq, user: self.uid.clone(),
			}),
		};
		let _ = tx.edit(req).await?;
		Ok(())
	}
}

#[async_trait]
impl ControllerWorker<TextChange> for BufferControllerWorker {
	type Controller = BufferController;
	type Tx = BufferClient<Channel>;
	type Rx = Streaming<RawOp>;

	fn subscribe(&self) -> BufferController {
		BufferController::new(
			self.receiver.clone(),
			self.sender.clone(),
			self.stop_control.clone(),
		)
	}

	async fn work(mut self, mut tx: Self::Tx, mut rx: Self::Rx) {
		loop {
			// block until one of these is ready
			tokio::select! {
				biased;

				// received stop signal
				_ = self.stop.recv() => break,

				// received a text change from editor
				res = self.operations.recv() => match res {
					None => break,
					Some(change) => {
						match self.buffer.view().get(change.span.clone()) {
							None =>  tracing::error!("received illegal span from client"),
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
									match self.send_op(&mut tx, &op).await {
										Err(e) => tracing::error!("server refused to broadcast {}: {}", op, e),
										Ok(()) => {
											self.content.send(self.buffer.view()).unwrap_or_warn("could not send buffer update");
										},
									}
								}
							},
						}
					}
				},

				// received a stop request (or channel got closed)
				res = rx.message() => match res {
					Err(_e) => break,
					Ok(None) => break,
					Ok(Some(change)) => match serde_json::from_str::<Op>(&change.opseq) {
						Ok(op) => {
							self.buffer.merge(op);
							self.content.send(self.buffer.view()).unwrap_or_warn("could not send buffer update");
						},
						Err(e) => tracing::error!("could not deserialize operation from server: {}", e),
					},
				},
			}

		}
	}
}
