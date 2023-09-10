use std::collections::VecDeque;

use operational_transform::OperationSeq;
use tokio::sync::{watch, mpsc, oneshot, Mutex};
use tonic::transport::Channel;
use tonic::{async_trait, Streaming};

use crate::proto::{OperationRequest, RawOp};
use crate::proto::buffer_client::BufferClient;
use crate::api::controller::ControllerWorker;
use crate::api::factory::{leading_noop, tailing_noop};

use super::TextChange;
use super::controller::BufferController;


pub(crate) struct BufferControllerWorker {
	uid: String,
	content: watch::Sender<String>,
	operations: mpsc::UnboundedReceiver<OperationSeq>,
	stream: mpsc::UnboundedReceiver<oneshot::Sender<Option<TextChange>>>,
	stream_requestor: mpsc::UnboundedSender<oneshot::Sender<Option<TextChange>>>,
	receiver: watch::Receiver<String>,
	sender: mpsc::UnboundedSender<OperationSeq>,
	buffer: String,
	path: String,
	stop: mpsc::UnboundedReceiver<()>,
	stop_control: mpsc::UnboundedSender<()>,
	new_op_tx: watch::Sender<()>,
	new_op_rx: watch::Receiver<()>,
}

impl BufferControllerWorker {
	pub fn new(uid: String, buffer: &str, path: &str) -> Self {
		let (txt_tx, txt_rx) = watch::channel(buffer.to_string());
		let (op_tx, op_rx) = mpsc::unbounded_channel();
		let (s_tx, s_rx) = mpsc::unbounded_channel();
		let (end_tx, end_rx) = mpsc::unbounded_channel();
		let (notx, norx) = watch::channel(());
		BufferControllerWorker {
			uid,
			content: txt_tx,
			operations: op_rx,
			stream: s_rx,
			stream_requestor: s_tx,
			receiver: txt_rx,
			sender: op_tx,
			buffer: buffer.to_string(),
			path: path.to_string(),
			stop: end_rx,
			stop_control: end_tx,
			new_op_tx: notx,
			new_op_rx: norx,
		}
	}

	async fn send_op(&self, tx: &mut BufferClient<Channel>, outbound: &OperationSeq) -> crate::Result<()> {
		let opseq = serde_json::to_string(outbound).expect("could not serialize opseq");
		let req = OperationRequest {
			path: self.path.clone(),
			hash: format!("{:x}", md5::compute(&self.buffer)),
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
			self.stream_requestor.clone(),
			self.stop_control.clone(),
			Mutex::new(self.new_op_rx.clone()),
		)
	}

	async fn work(mut self, mut tx: Self::Tx, mut rx: Self::Rx) {
		let mut clientside : VecDeque<OperationSeq> = VecDeque::new();
		let mut serverside : VecDeque<OperationSeq> = VecDeque::new();

		loop {

			// block until one of these is ready
			tokio::select! {
				biased;

				// received a stop request (or channel got closed)
				res = self.stop.recv() => {
					tracing::info!("received stop signal");
					match res {
						None => return tracing::warn!("stop channel closed, stopping worker"),
						Some(()) => return tracing::debug!("buffer worker stopping cleanly"),
					}
				}

				// received a new message from server (or an error)
				res = rx.message() => {
					tracing::info!("received msg from server");
					let inbound : OperationSeq = match res {
						Err(e) => return tracing::error!("error receiving op from server: {}", e),
						Ok(None) => return tracing::warn!("server closed operation stream"),
						Ok(Some(msg)) => serde_json::from_str(&msg.opseq)
								.expect("could not deserialize server opseq"),
					};
					self.buffer = inbound.apply(&self.buffer).expect("could not apply remote opseq???");
					serverside.push_back(inbound);
					while let Some(mut outbound) = clientside.get(0).cloned() {
						let mut serverside_tmp = serverside.clone();
						for server_op in serverside_tmp.iter_mut() {
							tracing::info!("transforming {:?} <-> {:?}", outbound, server_op);
							(outbound, *server_op) = outbound.transform(server_op)
								.expect("could not transform enqueued out with just received");
						}
						match self.send_op(&mut tx, &outbound).await {
							Err(e) => { tracing::warn!("could not send op even after transforming: {}", e); break; },
							Ok(()) => {
								tracing::info!("back in sync");
								serverside = serverside_tmp;
								self.buffer = outbound.apply(&self.buffer).expect("could not apply op after synching back");
								clientside.pop_front();
							},
						}
					}
					self.content.send(self.buffer.clone()).expect("could not broadcast buffer update");
					self.new_op_tx.send(()).expect("could not activate client after new server event");
				},

				// received a new operation from client (or channel got closed)
				res = self.operations.recv() => {
					tracing::info!("received op from client");
					match res {
						None => return tracing::warn!("client closed operation stream"),
						Some(op) => {
							if clientside.is_empty() {
								match self.send_op(&mut tx, &op).await {
									Ok(()) => {
										self.buffer = op.apply(&self.buffer).expect("could not apply op");
										self.content.send(self.buffer.clone()).expect("could not update buffer view");
									},
									Err(e) => {
										tracing::warn!("server rejected op: {}", e);
										clientside.push_back(op);
									},
								}
							} else { // I GET STUCK IN THIS BRANCH AND NOTHING HAPPENS AAAAAAAAAA
								clientside.push_back(op);
							}
						}
					}
				},
				
				// client requested a server operation, transform it and send it
				res = self.stream.recv() => {
					tracing::info!("received op REQUEST from client");
					match res {
						None => return tracing::error!("client closed requestor stream"),
						Some(tx) => tx.send(match serverside.pop_front() {
							None => {
								tracing::warn!("requested change but none is available");
								None
							},
							Some(mut operation) => {
								let mut after = self.buffer.clone();
								for op in clientside.iter_mut() {
									(*op, operation) = match op.transform(&operation) {
										Err(e) => return tracing::warn!("could not transform enqueued operation: {}", e),
										Ok((x, y)) => (x, y),
									};
									after = match op.apply(&after) {
										Err(_) => return tracing::error!("could not apply outgoing enqueued opseq to current buffer?"),
										Ok(x) => x,
									};
								}

								let skip = leading_noop(operation.ops()) as usize;
								let tail = tailing_noop(operation.ops()) as usize;
								let span = skip..(operation.base_len() - tail);
								let content = if after.len() - tail < skip { "".into() } else { after[skip..after.len()-tail].to_string() };
								let change = TextChange { span, content, after };

								Some(change)
							},
						}).expect("client did not wait????"),
					}
				},

			}
		}
	}
}
