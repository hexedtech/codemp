use std::sync::atomic::{AtomicU64, Ordering};
use std::{sync::Arc, collections::VecDeque};

use operational_transform::OperationSeq;
use tokio::sync::{watch, mpsc, broadcast, Mutex};
use tonic::transport::Channel;
use tonic::{async_trait, Streaming};

use crate::errors::IgnorableError;
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
	stream: Arc<broadcast::Sender<TextChange>>,
	receiver: watch::Receiver<String>,
	sender: mpsc::UnboundedSender<OperationSeq>,
	buffer: String,
	path: String,
	stop: mpsc::UnboundedReceiver<()>,
	stop_control: mpsc::UnboundedSender<()>,
	operation_tick: Arc<AtomicU64>,
}

impl BufferControllerWorker {
	pub fn new(uid: String, buffer: &str, path: &str) -> Self {
		let (txt_tx, txt_rx) = watch::channel(buffer.to_string());
		let (op_tx, op_rx) = mpsc::unbounded_channel();
		let (s_tx, _s_rx) = broadcast::channel(64);
		let (end_tx, end_rx) = mpsc::unbounded_channel();
		BufferControllerWorker {
			uid,
			content: txt_tx,
			operations: op_rx,
			stream: Arc::new(s_tx),
			receiver: txt_rx,
			sender: op_tx,
			buffer: buffer.to_string(),
			path: path.to_string(),
			stop: end_rx,
			stop_control: end_tx,
			operation_tick: Arc::new(AtomicU64::new(0)),
		}
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
			Mutex::new(self.stream.subscribe()),
			self.stop_control.clone(),
			self.operation_tick.clone(),
		)
	}

	async fn work(mut self, mut tx: Self::Tx, mut rx: Self::Rx) {
		let mut clientside : VecDeque<OperationSeq> = VecDeque::new();
		let mut serverside : VecDeque<OperationSeq> = VecDeque::new();
		let mut last_seen_tick = 0;

		loop {

			// block until one of these is ready
			tokio::select! {

				// received a new message from server (or an error)
				res = rx.message() => {
					match res {
						Err(e) => return tracing::error!("error receiving op from server: {}", e),
						Ok(None) => return tracing::warn!("server closed operation stream"),
						Ok(Some(msg)) => serverside.push_back(
							serde_json::from_str(&msg.opseq)
								.expect("could not deserialize server opseq")
						),
					}
				},

				// received a new operation from client (or channel got closed)
				res = self.operations.recv() => {
					match res {
						None => return tracing::warn!("client closed operation stream"),
						Some(op) => {
							clientside.push_back(op.clone());
							last_seen_tick = self.operation_tick.load(Ordering::Acquire);
						}
					}
				},

				// received a stop request (or channel got closed)
				res = self.stop.recv() => {
					match res {
						None => return tracing::warn!("stop channel closed, stopping worker"),
						Some(()) => return tracing::debug!("buffer worker stopping cleanly"),
					}
				}

			}

			// we must give priority to operations received from remote server, because we can transform
			// our ops with server's ops but server won't transform its ops with ours. We must transform
			// ALL enqueued client ops: if a new one arrived before we could transform and update, we
			// should discard our progress and poll again.
			while let Some(operation) = serverside.get(0).cloned() {
				let mut transformed_op = operation.clone();
				let mut queued_ops = clientside.clone();
				let mut txt_before = self.buffer.clone();
				for op in queued_ops.iter_mut() {
					txt_before = match op.apply(&txt_before) {
						Ok(x) => x,
						Err(_) => { tracing::error!("could not apply outgoing enqueued opseq to current buffer?"); break; },
					};
					(*op, transformed_op) = match op.transform(&transformed_op) {
						Err(e) => { tracing::warn!("could not transform enqueued operation: {}", e); break; },
						Ok((x, y)) => (x, y),
					};
				}

				let skip = leading_noop(transformed_op.ops()) as usize;
				let tail = tailing_noop(transformed_op.ops()) as usize;
				let span = skip..(transformed_op.base_len() - tail);
				let after = transformed_op.apply(&txt_before).expect("could not apply transformed op");
				let change = TextChange { span, content: after[skip..after.len()-tail].to_string() };

				let tick = self.operation_tick.load(std::sync::atomic::Ordering::Acquire);
				if tick != last_seen_tick {
					tracing::warn!("skipping downstream because there are ops");
					break
				} // there are more ops to see first
				clientside = queued_ops;
				self.buffer = match operation.apply(&self.buffer) {
					Ok(x) => x,
					Err(_) => { tracing::error!("wtf received op could not be applied?"); break; },
				};
				if clientside.is_empty() {
					self.content.send(self.buffer.clone()).expect("could not broadcast new buffer content");
				}
				self.stream.send(change)
					.unwrap_or_warn("could not send operation to server");
				serverside.pop_front();
			}

			// if there are still serverside operations to be applied, we can't dispatch our local ones
			// yet because we need them to transform the ones sent by the server before applying them on
			// our local buffer. We may get here if a new local operation arrived before we could process
			// and transform all received server ops. since the buffer is different, it isn't safe to
			// apply them and we must transform them again. If the loop above left its queue not empty,
			// we should be guaranteed to unblock immediately in the select above because we have a new
			// client operation waiting for us to be enqueued
			if serverside.is_empty() {
				while let Some(op) = clientside.get(0) {
					let opseq = serde_json::to_string(&op).expect("could not serialize opseq");
					let req = OperationRequest {
						path: self.path.clone(),
						hash: format!("{:x}", md5::compute(&self.buffer)),
						op: Some(RawOp {
							opseq, user: self.uid.clone(),
						}),
					};
					if let Err(e) = tx.edit(req).await {
						tracing::warn!("server rejected operation: {}", e);
						break;
					}
					self.buffer = match op.apply(&self.buffer) {
						Ok(x) => x,
						Err(_) => { tracing::error!("wtf accepted remote op could not be applied to our buffer????"); break; },
					};
					self.content.send(self.buffer.clone()).expect("could not broadcast buffer update");
					clientside.pop_front();
				}
			} else {
				tracing::warn!("skipping upstream because there are ops");
			}
		}
	}
}
