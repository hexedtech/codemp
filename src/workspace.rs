use std::{collections::BTreeMap, ops::Range};

use tokio::sync::RwLock;
use tonic::Status;

use crate::{
	client::CodempClient, factory::OperationFactory,
	controller::{buffer::{OperationControllerHandle, OperationControllerSubscriber},
	cursor::{CursorControllerHandle, CursorSubscriber}}, proto::Cursor,
};

pub struct Workspace {
	client: CodempClient,
	buffers: RwLock<BTreeMap<String, OperationControllerHandle>>,
	cursor: RwLock<CursorControllerHandle>,
}

impl Workspace {
	pub async fn new(mut client: CodempClient) -> Result<Self, Status> {
		Ok(
			Workspace {
				cursor: RwLock::new(client.listen().await?),
				client,
				buffers: RwLock::new(BTreeMap::new()),
			}
		)
	}

	pub async fn create(&self, path: &str, content: Option<String>) -> Result<bool, Status> {
		self.client.clone().create(path.into(), content).await
	}

	pub async fn attach(&self, path: String) -> Result<(), Status> {
		self.buffers.write().await.insert(
			path.clone(),
			self.client.clone().attach(path).await?
		);
		Ok(())
	}

	pub async fn diff(&self, path: &str, span: Range<usize>, text: &str) {
		if let Some(controller) = self.buffers.read().await.get(path) {
			if let Some(op) = controller.delta(span.start, text, span.end) {
				controller.apply(op).await
			}
		}
	}

	pub async fn send(&self, path: &str, start: (i32, i32), end: (i32, i32)) {
		self.cursor.read().await.send(path, start.into(), end.into()).await
	}

	pub async fn recv(&self) -> Option<Cursor> {
		self.cursor.write().await.poll().await

	}
}
