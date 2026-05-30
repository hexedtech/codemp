use std::sync::Arc;

use tokio::sync::{mpsc, oneshot, watch};
use tonic::Streaming;

use crate::api::BufferUpdate;
use crate::api::TextChange;
use crate::api::controller::ControllerCallback;
use crate::ext::IgnorableError;

use codemp_proto::buffer::BufferEvent;

use super::controller::{BufferController, BufferControllerInner};

struct BufferWorker<T: crate::api::CRDT> {
	agent_id: T::AgentID,
	path: String,
	workspace_id: crate::proto::session::WorkspaceIdentifier,
	latest_version: watch::Sender<T::Version>,
	local_version: watch::Sender<T::Version>,
	ack_rx: mpsc::UnboundedReceiver<T::Version>,
	ops_in: mpsc::UnboundedReceiver<TextChange>,
	poller: mpsc::UnboundedReceiver<oneshot::Sender<()>>,
	pollers: Vec<oneshot::Sender<()>>,
	content_checkout: mpsc::Receiver<oneshot::Sender<String>>,
	delta_req: mpsc::Receiver<oneshot::Sender<Option<BufferUpdate>>>,
	controller: std::sync::Weak<BufferControllerInner<T>>,
	callback: watch::Receiver<Option<ControllerCallback<BufferController<T>>>>,
	oplog: T,
	branch: T::Version,
	timer: Timer,
}

impl<T: crate::api::CRDT<Location = usize> + Send + 'static> BufferController<T> {
	pub(crate) fn spawn(
		user_name: String,
		path: String,
		tx: mpsc::Sender<crate::proto::buffer::Operation>,
		rx: Streaming<BufferEvent>,
		workspace_id: crate::proto::session::WorkspaceIdentifier,
	) -> Self {
		let init = T::Version::default();

		let (latest_version_tx, latest_version_rx) = watch::channel(init.clone());
		let (my_version_tx, my_version_rx) = watch::channel(init.clone());
		let (opin_tx, opin_rx) = mpsc::unbounded_channel();
		let (ack_tx, ack_rx) = mpsc::unbounded_channel();

		let (req_tx, req_rx) = mpsc::channel(1);
		let (recv_tx, recv_rx) = mpsc::channel(1);
		let (cb_tx, cb_rx) = watch::channel(None);

		let (poller_tx, poller_rx) = mpsc::unbounded_channel();
		let mut oplog = T::default();
		let agent_id = oplog.agent(&user_name);

		let controller = Arc::new(BufferControllerInner {
			path: path.clone(),
			latest_version: latest_version_rx,
			local_version: my_version_rx,
			ops_in: opin_tx,
			poller: poller_tx,
			content_request: req_tx,
			delta_request: recv_tx,
			callback: cb_tx,
			ack_tx,
			workspace_id: workspace_id.clone(),
		});

		let weak = Arc::downgrade(&controller);

		let worker = BufferWorker {
			agent_id,
			path,
			workspace_id,
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
			oplog,
			branch: T::Version::default(),
			timer: Timer::new(10), // TODO configurable!
		};

		tokio::spawn(async move { BufferController::work(worker, tx, rx).await });

		BufferController(controller)
	}

	#[tracing::instrument(skip(worker, tx, rx), fields(owner = worker.workspace_id.user, ws = worker.workspace_id.workspace, path = worker.path))]
	async fn work(
		mut worker: BufferWorker<T>,
		tx: mpsc::Sender<crate::proto::buffer::Operation>,
		mut rx: Streaming<BufferEvent>,
	) {
		tracing::debug!("buffer worker started");
		loop {
			if worker.controller.upgrade().is_none() {
				break tracing::debug!("buffer worker clean exit");
			}

			// block until one of these is ready
			tokio::select! {
				biased;

				// received new change ack, merge editor branch up to that version
				res = worker.ack_rx.recv() => match res {
					None => break tracing::debug!("stopping: ack channel closed"),
					Some(v) => {
						tracing::debug!("client acked change");
						worker.branch = v;
						worker.local_version.send(worker.branch.clone())
							.unwrap_or_warn("could not checkout local version");
					},
				},

				// received a new poller, add it to collection
				res = worker.poller.recv() => match res {
					None => break tracing::debug!("stopping: poller channel closed"),
					Some(tx) => worker.pollers.push(tx),
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
					Some(tx) => worker.handle_delta_request(tx).await,
				},

				// received a request for full CRDT content
				res = worker.content_checkout.recv() => match res {
					None => break tracing::error!("no more active controllers: can't update content"),
					Some(tx) => {
						worker.branch = worker.oplog.version(); // consider everything checked-out from now
						worker.local_version.send(worker.branch.clone())
							.unwrap_or_warn("could not checkout local version");
						tx.send(worker.oplog.view())
							.unwrap_or_warn("checkout request dropped");
					},
				},

				_ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {},
			}
		}

		tracing::debug!("buffer worker stopped");
	}
}

impl<T: crate::api::CRDT<Location = usize>> BufferWorker<T> {
	#[tracing::instrument(skip(self, tx))]
	async fn handle_editor_change(&mut self, change: TextChange, tx: &mpsc::Sender<crate::proto::buffer::Operation>) {
		let current_version = self.branch.clone();
		// clip to buffer extents
		let clip_start = change.start_idx as usize;
		let clip_end = change.end_idx as usize;

		//let b_len = self.branch.len();
		//if clip_end > b_len {
			//tracing::warn!("clipping TextChange end span from {clip_end} to {b_len}");
			//clip_end = b_len;
		//};

		// in case we have a "replace" span
		if change.is_delete() {
			let _ = self.oplog.delete_at(self.agent_id.clone(), clip_start, current_version.clone(), clip_end);
		}

		if change.is_insert() {
			let _ = self.oplog.insert_at(self.agent_id.clone(), clip_start, current_version.clone(), &change.content);
		}

		let last_ver = self.oplog.version();

		if change.is_delete() || change.is_insert() {
			let diff = self.oplog.diff(current_version.clone(), last_ver.clone());
			let op = crate::proto::buffer::Operation { data: diff.as_ref().to_vec() };
			tx.send(op)
			.await
			.unwrap_or_warn("failed to send change!");
			self.latest_version
				.send(last_ver)
				.unwrap_or_warn("failed to update latest version!");
			self.local_version
				.send(current_version)
				.unwrap_or_warn("failed to update local version!");
		}
	}

	#[tracing::instrument(skip(self, change), fields(user = change.user))]
	async fn handle_server_change(&mut self, change: BufferEvent) -> bool {
		match self.controller.upgrade() {
			None => {
				// clean exit actually, just weird we caught it here
				tracing::debug!("clean exit while handling server change");
				true
			}
			Some(controller) => match self.oplog.integrate(T::Diff::try_from(change.op.data).unwrap()) {
				Ok(()) => {
					tracing::debug!("updating local version: {:?}", self.oplog.version());
					self.latest_version
						.send(self.oplog.version())
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

	#[tracing::instrument(skip(self, tx))]
	async fn handle_delta_request(&mut self, tx: oneshot::Sender<Option<BufferUpdate>>) {
		let starting_ver = self.branch.clone();
		let last_ver = self.oplog.version();

		let mut ops = Vec::new();

		for (span, content) in self
			.oplog
			.diff(starting_ver.clone(), last_ver.clone())
		{
			// x.0.start should always be after lastver!
			// this step_ver will be the version after we apply the operation
			// we give it to the controller so that he knows where it's at.
			// TODO do we still need this?
			//let step_ver = self.oplog.version_union(&[lv.end - 1], &last_ver);

			ops.push(crate::api::TextChange {
				start_idx: span.start as u32,
				end_idx: span.end as u32,
				content,
			});
		}

		if ops.is_empty() {
			tx.send(None)
				.unwrap_or_warn("could not update ops channel -- is controller dead?");
			return;
		}

		let hash = if self.timer.step() {
			Some(crate::ext::hash(self.oplog.view_at(starting_ver.clone())))
		} else {
			None
		};

		let tc = crate::api::BufferUpdate {
			hash,
			version: crate::api::crdt::translate_version::<T>(last_ver.clone()),
			changes: ops,
		};

		tracing::debug!("sending update {:?}", tc.clone());
		tx.send(Some(tc))
			.unwrap_or_warn("could not update ops channel -- is controller dead?");
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
