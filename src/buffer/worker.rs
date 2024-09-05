use std::sync::Arc;

use diamond_types::LocalVersion;
use tokio::sync::{mpsc, oneshot, watch};
use tonic::Streaming;
use uuid::Uuid;

use crate::api::controller::{ControllerCallback, ControllerWorker};
use crate::api::TextChange;
use crate::ext::{IgnorableError, InternallyMutable};

use codemp_proto::buffer::{BufferEvent, Operation};

use super::controller::{BufferController, BufferControllerInner};

pub(crate) type DeltaOp = (LocalVersion, Option<TextChange>);
pub(crate) type DeltaRequest = (LocalVersion, oneshot::Sender<DeltaOp>);

pub(crate) struct BufferWorker {
	user_id: Uuid,
	latest_version: watch::Sender<diamond_types::LocalVersion>,
	ops_in: mpsc::UnboundedReceiver<(TextChange, oneshot::Sender<LocalVersion>)>,
	poller: mpsc::UnboundedReceiver<oneshot::Sender<()>>,
	pollers: Vec<oneshot::Sender<()>>,
	content_checkout: mpsc::Receiver<oneshot::Sender<String>>,
	delta_req: mpsc::Receiver<DeltaRequest>,
	stop: mpsc::UnboundedReceiver<()>,
	controller: BufferController,
	callback: watch::Receiver<Option<ControllerCallback<BufferController>>>,
}

impl BufferWorker {
	pub fn new(user_id: Uuid, path: &str) -> Self {
		let init = diamond_types::LocalVersion::default();

		let (latest_version_tx, latest_version_rx) = watch::channel(init.clone());
		let (opin_tx, opin_rx) = mpsc::unbounded_channel();

		let (req_tx, req_rx) = mpsc::channel(1);
		let (recv_tx, recv_rx) = mpsc::channel(1);
		let (cb_tx, cb_rx) = watch::channel(None);

		let (poller_tx, poller_rx) = mpsc::unbounded_channel();

		let (end_tx, end_rx) = mpsc::unbounded_channel();

		let controller = BufferControllerInner {
			name: path.to_string(),
			latest_version: latest_version_rx,
			last_update: InternallyMutable::new(diamond_types::LocalVersion::default()),
			ops_in: opin_tx,
			poller: poller_tx,
			stopper: end_tx,
			content_request: req_tx,
			delta_request: recv_tx,
			callback: cb_tx,
		};

		BufferWorker {
			user_id,
			latest_version: latest_version_tx,
			ops_in: opin_rx,
			poller: poller_rx,
			pollers: Vec::new(),
			stop: end_rx,
			controller: BufferController(Arc::new(controller)),
			content_checkout: req_rx,
			delta_req: recv_rx,
			callback: cb_rx,
		}
	}
}

impl ControllerWorker<TextChange> for BufferWorker {
	type Controller = BufferController;
	type Tx = mpsc::Sender<Operation>;
	type Rx = Streaming<BufferEvent>;

	fn controller(&self) -> BufferController {
		self.controller.clone()
	}

	async fn work(mut self, tx: Self::Tx, mut rx: Self::Rx) {
		let mut branch = diamond_types::list::Branch::new();
		let mut oplog = diamond_types::list::OpLog::new();
		let mut timer = Timer::new(10); // TODO configurable!!
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
				res = self.ops_in.recv() => match res {
					None => break tracing::debug!("stopping: editor closed channel"),
					Some((change, ack)) => {
						let agent_id = oplog.get_or_create_agent_id(&self.user_id.to_string());
						let last_ver = oplog.local_version();
						// clip to buffer extents
						let clip_end = std::cmp::min(branch.len(), change.end as usize);
						let clip_start = std::cmp::max(0, change.start as usize);

						// in case we have a "replace" span
						if change.is_delete() {
							branch.delete_without_content(&mut oplog, agent_id, clip_start..clip_end);
						}

						if change.is_insert() {
							branch.insert(&mut oplog, agent_id, clip_start, &change.content);
						}

						if change.is_delete() || change.is_insert() {
							tx.send(Operation { data: oplog.encode_from(Default::default(), &last_ver) }).await
								.unwrap_or_warn("failed to send change!");
							self.latest_version.send(oplog.local_version())
								.unwrap_or_warn("failed to update latest version!");
						}
						ack.send(branch.local_version()).unwrap_or_warn("controller didn't wait for ack");
					},
				},

				// received a message from server: add to oplog and update latest version (+unlock pollers)
				res = rx.message() => match res {
					Err(_e) => break,
					Ok(None) => break,
					Ok(Some(change)) => {
						match oplog.decode_and_add(&change.op.data) {
							Ok(local_version) => {
								self.latest_version.send(local_version)
									.unwrap_or_warn("failed to update latest version!");
								for tx in self.pollers.drain(..) {
									tx.send(()).unwrap_or_warn("could not wake up poller");
								}
								if let Some(cb) = self.callback.borrow().as_ref() {
									cb.call(self.controller.clone()); // TODO should we run this on another task/thread?
								}
							},
							Err(e) => tracing::error!("could not deserialize operation from server: {}", e),
						}
					},
				},

				// controller is ready to apply change and recv(), calculate it and send it back
				res = self.delta_req.recv() => match res {
					None => break tracing::error!("no more active controllers: can't send changes"),
					Some((last_ver, tx)) => {
						if let Some((lv, Some(dtop))) = oplog.iter_xf_operations_from(&last_ver, oplog.local_version_ref()).next() {
							// x.0.start should always be after lastver!
							// this step_ver will be the version after we apply the operation
							// we give it to the controller so that he knows where it's at.
							let step_ver = oplog.version_union(&[lv.end-1], &last_ver);
							branch.merge(&oplog, &step_ver);
							let new_local_v = branch.local_version();

							let hash = if timer.step() {
								Some(crate::ext::hash(branch.content().to_string()))
							} else { None };

							let tc = match dtop.kind {
								diamond_types::list::operation::OpKind::Ins => {
									if dtop.end() - dtop.start() != dtop.content_as_str().unwrap_or_default().len() {
										tracing::error!("[?!?!] Insert span differs from effective content len (TODO remove this error after a bit)");
									}
									crate::api::change::TextChange {
										start: dtop.start() as u32,
										end: dtop.start() as u32,
										content: dtop.content_as_str().unwrap_or_default().to_string(),
										hash
									}
								},

								diamond_types::list::operation::OpKind::Del => {
									crate::api::change::TextChange {
										start: dtop.start() as u32,
										end: dtop.end() as u32,
										content: dtop.content_as_str().unwrap_or_default().to_string(),
										hash
									}
								}
							};
							tx.send((new_local_v, Some(tc))).unwrap_or_warn("could not update ops channel -- is controller dead?");
						} else {
							tx.send((last_ver, None)).unwrap_or_warn("could not update ops channel -- is controller dead?");
						}
					},
				},

				// received a request for full CRDT content
				res = self.content_checkout.recv() => match res {
					None => break tracing::error!("no more active controllers: can't update content"),
					Some(tx) => {
						branch.merge(&oplog, oplog.local_version_ref());
						let content = branch.content().to_string();
						tx.send(content).unwrap_or_warn("checkout request dropped");
					},
				}
			}
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
