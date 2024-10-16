use std::sync::Arc;

use diamond_types::list::{Branch, OpLog};
use diamond_types::LocalVersion;
use tokio::sync::{mpsc, oneshot, watch};
use tonic::Streaming;
use uuid::Uuid;

use crate::api::controller::ControllerCallback;
use crate::api::BufferUpdate;
use crate::api::TextChange;
use crate::ext::IgnorableError;

use codemp_proto::buffer::{BufferEvent, Operation};

use super::controller::{BufferController, BufferControllerInner};

struct BufferWorker {
	agent_id: u32,
	path: String,
	latest_version: watch::Sender<diamond_types::LocalVersion>,
	local_version: watch::Sender<diamond_types::LocalVersion>,
	ack_rx: mpsc::UnboundedReceiver<LocalVersion>,
	ops_in: mpsc::UnboundedReceiver<TextChange>,
	poller: mpsc::UnboundedReceiver<oneshot::Sender<()>>,
	pollers: Vec<oneshot::Sender<()>>,
	content_checkout: mpsc::Receiver<oneshot::Sender<String>>,
	delta_req: mpsc::Receiver<(LocalVersion, oneshot::Sender<Option<BufferUpdate>>)>,
	controller: std::sync::Weak<BufferControllerInner>,
	callback: watch::Receiver<Option<ControllerCallback<BufferController>>>,
	oplog: OpLog,
	branch: Branch,
	timer: Timer,
}

impl BufferController {
	pub(crate) fn spawn(
		user_id: Uuid,
		path: &str,
		tx: mpsc::Sender<Operation>,
		rx: Streaming<BufferEvent>,
	) -> Self {
		let init = diamond_types::LocalVersion::default();

		let (latest_version_tx, latest_version_rx) = watch::channel(init.clone());
		let (my_version_tx, my_version_rx) = watch::channel(init.clone());
		let (opin_tx, opin_rx) = mpsc::unbounded_channel();
		let (ack_tx, ack_rx) = mpsc::unbounded_channel();

		let (req_tx, req_rx) = mpsc::channel(1);
		let (recv_tx, recv_rx) = mpsc::channel(1);
		let (cb_tx, cb_rx) = watch::channel(None);

		let (poller_tx, poller_rx) = mpsc::unbounded_channel();
		let mut oplog = OpLog::new();
		let agent_id = oplog.get_or_create_agent_id(&user_id.to_string());

		let controller = Arc::new(BufferControllerInner {
			name: path.to_string(),
			latest_version: latest_version_rx,
			local_version: my_version_rx,
			ops_in: opin_tx,
			poller: poller_tx,
			content_request: req_tx,
			delta_request: recv_tx,
			callback: cb_tx,
			ack_tx,
		});

		let weak = Arc::downgrade(&controller);

		let worker = BufferWorker {
			agent_id,
			path: path.to_string(),
			latest_version: latest_version_tx,
			local_version: my_version_tx,
			ack_rx,
			ops_in: opin_rx,
			poller: poller_rx,
			pollers: Vec::new(),
			controller: weak,
			content_checkout: req_rx,
			delta_req: recv_rx,
			callback: cb_rx,
			oplog: OpLog::new(),
			branch: Branch::new(),
			timer: Timer::new(10), // TODO configurable!
		};

		tokio::spawn(async move { BufferController::work(worker, tx, rx).await });

		BufferController(controller)
	}

	async fn work(
		mut worker: BufferWorker,
		tx: mpsc::Sender<Operation>,
		mut rx: Streaming<BufferEvent>,
	) {
		tracing::debug!("controller worker started");
		loop {
			if worker.controller.upgrade().is_none() {
				break;
			};

			// block until one of these is ready
			tokio::select! {
				biased;

				// received a new poller, add it to collection
				res = worker.poller.recv() => match res {
					None => break tracing::error!("poller channel closed"),
					Some(tx) => worker.pollers.push(tx),
				},

				// received new change ack, merge editor branch up to that version
				res = worker.ack_rx.recv() => match res {
					None => break tracing::error!("ack channel closed"),
					Some(v) => {
						worker.branch.merge(&worker.oplog, &v)
					},
				},

				// received a text change from editor
				res = worker.ops_in.recv() => match res {
					None => break tracing::debug!("stopping: editor closed channel"),
					Some(change) => worker.handle_editor_change(change, &tx).await,
				},

				// received a message from server: add to oplog and update latest version (+unlock pollers)
				res = rx.message() => match res {
					Err(e) => break tracing::warn!("error receiving from server for buffer {}: {e}", worker.path),
					Ok(None) => break tracing::info!("disconnected from buffer {}", worker.path),
					Ok(Some(change)) => if worker.handle_server_change(change).await { break },
				},

				// controller is ready to apply change and recv(), calculate it and send it back
				res = worker.delta_req.recv() => match res {
					None => break tracing::error!("no more active controllers: can't send changes"),
					Some((last_ver, tx)) => worker.handle_delta_request(last_ver, tx).await,
				},

				// received a request for full CRDT content
				res = worker.content_checkout.recv() => match res {
					None => break tracing::error!("no more active controllers: can't update content"),
					Some(tx) => {
						worker.branch.merge(&worker.oplog, worker.oplog.local_version_ref());
						let content = worker.branch.content().to_string();
						tx.send(content).unwrap_or_warn("checkout request dropped");
					},
				}
			}
		}

		tracing::debug!("controller worker stopped");
	}
}

impl BufferWorker {
	async fn handle_editor_change(&mut self, change: TextChange, tx: &mpsc::Sender<Operation>) {
		let last_ver = self.oplog.local_version();
		// clip to buffer extents
		let clip_start = change.start_idx as usize;
		let mut clip_end = change.end_idx as usize;
		let b_len = self.branch.len();
		if clip_end > b_len {
			tracing::warn!("clipping TextChange end span from {clip_end} to {b_len}");
			clip_end = b_len;
		};

		// in case we have a "replace" span
		if change.is_delete() {
			self.branch.delete_without_content(
				&mut self.oplog,
				self.agent_id,
				clip_start..clip_end,
			);
		}

		if change.is_insert() {
			self.branch
				.insert(&mut self.oplog, self.agent_id, clip_start, &change.content);
		}

		if change.is_delete() || change.is_insert() {
			tx.send(Operation {
				data: self.oplog.encode_from(Default::default(), &last_ver),
			})
			.await
			.unwrap_or_warn("failed to send change!");
			self.latest_version
				.send(self.oplog.local_version())
				.unwrap_or_warn("failed to update latest version!");
			self.local_version
				.send(self.branch.local_version())
				.unwrap_or_warn("failed to update local version!");
		}
	}

	async fn handle_server_change(&mut self, change: BufferEvent) -> bool {
		match self.controller.upgrade() {
			None => true, // clean exit actually, just weird we caught it here
			Some(controller) => match self.oplog.decode_and_add(&change.op.data) {
				Ok(local_version) => {
					self.latest_version
						.send(local_version)
						.unwrap_or_warn("failed to update latest version!");
					for tx in self.pollers.drain(..) {
						tx.send(()).unwrap_or_warn("could not wake up poller");
					}
					if let Some(cb) = self.callback.borrow().as_ref() {
						cb.call(BufferController(controller)); // TODO should we run this on another task/thread?
					}
					false
				}
				Err(e) => {
					tracing::error!("could not deserialize operation from server: {}", e);
					true
				}
			},
		}
	}

	async fn handle_delta_request(
		&mut self,
		last_ver: LocalVersion,
		tx: oneshot::Sender<Option<BufferUpdate>>,
	) {
		if let Some((lv, Some(dtop))) = self
			.oplog
			.iter_xf_operations_from(&last_ver, self.oplog.local_version_ref())
			.next()
		{
			// x.0.start should always be after lastver!
			// this step_ver will be the version after we apply the operation
			// we give it to the controller so that he knows where it's at.
			let step_ver = self.oplog.version_union(&[lv.end - 1], &last_ver);
			self.branch.merge(&self.oplog, &step_ver);
			let new_local_v = self.branch.local_version();

			let hash = if self.timer.step() {
				Some(crate::ext::hash(self.branch.content().to_string()))
			} else {
				None
			};

			let tc = match dtop.kind {
				diamond_types::list::operation::OpKind::Ins => {
					if dtop.end() - dtop.start() != dtop.content_as_str().unwrap_or_default().len()
					{
						tracing::warn!(
							"Insert span ({}, {}) differs from effective content len ({})",
							dtop.start(),
							dtop.end(),
							dtop.content_as_str().unwrap_or_default().len()
						);
					}
					crate::api::BufferUpdate {
						hash,
						version: step_ver
							.into_iter()
							.map(|x| i64::from_ne_bytes(x.to_ne_bytes()))
							.collect(), // TODO this is wasteful
						change: crate::api::TextChange {
							start_idx: dtop.start() as u32,
							end_idx: dtop.start() as u32,
							content: dtop.content_as_str().unwrap_or_default().to_string(),
						},
					}
				}

				diamond_types::list::operation::OpKind::Del => crate::api::BufferUpdate {
					hash,
					version: step_ver
						.into_iter()
						.map(|x| i64::from_ne_bytes(x.to_ne_bytes()))
						.collect(), // TODO this is wasteful
					change: crate::api::TextChange {
						start_idx: dtop.start() as u32,
						end_idx: dtop.end() as u32,
						content: dtop.content_as_str().unwrap_or_default().to_string(),
					},
				},
			};
			self.local_version
				.send(new_local_v)
				.unwrap_or_warn("could not update local version");
			tx.send(Some(tc))
				.unwrap_or_warn("could not update ops channel -- is controller dead?");
		} else {
			tx.send(None)
				.unwrap_or_warn("could not update ops channel -- is controller dead?");
		}
	}
}

struct Timer(u32, u32);
impl Timer {
	fn new(period: u32) -> Self {
		Timer(0, period)
	}
	fn step(&mut self) -> bool {
		self.0 += 1;
		if self.0 >= self.1 {
			self.0 = 0;
			true
		} else {
			false
		}
	}
}
