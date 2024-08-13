use std::sync::Arc;

use diamond_types::LocalVersion;
use tokio::sync::{mpsc, oneshot, watch};
use tonic::{async_trait, Streaming};
use uuid::Uuid;

use crate::api::controller::ControllerWorker;
use crate::api::Op;
use crate::api::TextChange;

use crate::errors::IgnorableError;
use codemp_proto::buffer::{BufferEvent, Operation};

use super::controller::{BufferController, BufferControllerInner};

pub(crate) struct BufferWorker {
	user_id: Uuid,
	latest_version: watch::Sender<diamond_types::LocalVersion>,
	ops_in: mpsc::UnboundedReceiver<TextChange>,
	ops_out: mpsc::UnboundedSender<(LocalVersion, Option<Op>)>,
	poller: mpsc::UnboundedReceiver<oneshot::Sender<()>>,
	pollers: Vec<oneshot::Sender<()>>,
	content_checkout: mpsc::Receiver<oneshot::Sender<String>>,
	stop: mpsc::UnboundedReceiver<()>,
	controller: BufferController,
}

impl BufferWorker {
	pub fn new(user_id: Uuid, path: &str) -> Self {
		let init = diamond_types::LocalVersion::default();

		let (latest_version_tx, latest_version_rx) = watch::channel(init.clone());
		let (opin_tx, opin_rx) = mpsc::unbounded_channel();
		let (opout_tx, opout_rx) = mpsc::unbounded_channel();

		let (req_tx, req_rx) = mpsc::channel(1);

		let (poller_tx, poller_rx) = mpsc::unbounded_channel();

		let (end_tx, end_rx) = mpsc::unbounded_channel();

		let controller = BufferControllerInner::new(
			path.to_string(),
			latest_version_rx,
			opin_tx,
			opout_rx,
			poller_tx,
			end_tx,
			req_tx,
		);

		BufferWorker {
			user_id,
			latest_version: latest_version_tx,
			ops_in: opin_rx,
			ops_out: opout_tx,
			poller: poller_rx,
			pollers: Vec::new(),
			stop: end_rx,
			controller: BufferController(Arc::new(controller)),
			content_checkout: req_rx,
		}
	}
}

#[async_trait]
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
					Some(change) => {
						let agent_id = oplog.get_or_create_agent_id(&self.user_id.to_string());
						let last_ver = oplog.local_version();

						if change.is_insert() {
							oplog.add_insert(agent_id, change.start as usize, &change.content)
						} else if change.is_delete() {
							oplog.add_delete_without_content(1, change.span())
						} else { continue; };

						tx.send(Operation { data: oplog.encode_from(Default::default(), &last_ver) }).await
							.unwrap_or_warn("failed to send change!");
						self.latest_version.send(oplog.local_version())
							.unwrap_or_warn("failed to update latest version!");

					},
				},

				// received a message from server
				res = rx.message() => match res {
					Err(_e) => break,
					Ok(None) => break,
					Ok(Some(change)) => {
						let last_ver = oplog.local_version();
						match oplog.decode_and_add(&change.op.data) {
							Ok(local_version) => {
								// give all the changes needed to the controller in a channel.
								for (lv, dtop) in oplog.iter_xf_operations_from(&last_ver, &local_version) {
									if let Some(dtop) = dtop {
										// x.0.start should always be after lastver!
										// this step_ver will be the version after we apply the operation
										// we give it to the controller so that he knows where it's at.
										let step_ver = oplog.version_union(&[lv.start], &last_ver);
										let opout = (step_ver, Some(Op(dtop)));

										self.ops_out.send(opout).unwrap_or_warn("could not update ops channel -- is controller dead?");
									}
								}

								// finally we send the 
								self.latest_version.send(local_version)
									.unwrap_or_warn("failed to update latest version!");

								for tx in self.pollers.drain(..) {
									tx.send(()).unwrap_or_warn("could not wake up poller");
								}
							},
							Err(e) => tracing::error!("could not deserialize operation from server: {}", e),
						}
					},
				},

				res = self.content_checkout.recv() => match res {
					None => break tracing::error!("no more active controllers"),
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
