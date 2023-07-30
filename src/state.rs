use std::{collections::BTreeMap, sync::Arc};

use tokio::sync::RwLock;
use tonic::async_trait;

use crate::{
	buffer::{client::CodempClient, controller::OperationControllerSubscriber},
	cursor::controller::CursorSubscriber, errors::CodempError,
};

#[cfg(feature = "static")]
pub mod instance {
	use tokio::runtime::Runtime;
	use super::Workspace;

	const CODEMP_DEFAULT_HOST : &str = "http://fantabos.co:50051";

	lazy_static::lazy_static! {
		static ref RUNTIME   : Runtime = Runtime::new().expect("could not create tokio runtime");
		static ref WORKSPACE : Workspace = RUNTIME.block_on(
			Workspace::new(&std::env::var("CODEMP_HOST").unwrap_or(CODEMP_DEFAULT_HOST.into()))
		).expect("could not create codemp workspace");
	}
}

pub struct Workspace {
	client: CodempClient,
	buffers: RwLock<BTreeMap<Box<str>, BufferController>>,
	cursor: CursorController,
}

pub type CursorController = Arc<dyn CursorSubscriber + Send + Sync>;
pub type BufferController = Arc<dyn OperationControllerSubscriber + Send + Sync>;

#[async_trait]
pub trait WorkspaceHandle {
	async fn cursor(&self) -> CursorController;
	async fn buffer(&self, path: &str) -> Option<BufferController>;
	async fn attach(&self, path: &str) -> Result<(), CodempError>;
	async fn create(&self, path: &str, content: Option<&str>) -> Result<bool, CodempError>;
}

impl Workspace {
	pub async fn new(dest: &str) -> Result<Self, CodempError> {
		let mut client = CodempClient::new(dest).await?;
		let cursor = Arc::new(client.listen().await?);
		Ok(
			Workspace {
				buffers: RwLock::new(BTreeMap::new()),
				cursor,
				client,
			}
		)
	}
}

#[async_trait]
impl WorkspaceHandle for Workspace {
	// Cursor
	async fn cursor(&self) -> CursorController {
		self.cursor.clone()
	}

	// Buffer
	async fn buffer(&self, path: &str) -> Option<BufferController> {
		self.buffers.read().await.get(path).cloned()
	}

	async fn create(&self, path: &str, content: Option<&str>) -> Result<bool, CodempError> {
		Ok(self.client.clone().create(path, content).await?)
	}

	async fn attach(&self, path: &str) -> Result<(), CodempError> {
		let controller = self.client.clone().attach(path).await?;
		self.buffers.write().await.insert(path.into(), Arc::new(controller));
		Ok(())
	}

	// pub async fn diff(&self, path: &str, span: Range<usize>, text: &str) {
	// 	if let Some(controller) = self.inner.read().await.buffers.get(path) {
	// 		if let Some(op) = controller.delta(span.start, text, span.end) {
	// 			controller.apply(op).await
	// 		}
	// 	}
	// }

	// async fn send(&self, path: &str, start: (i32, i32), end: (i32, i32)) {
	// 	self.inner.read().await.cursor.send(path, start.into(), end.into()).await
	// }

	// pub async fn recv(&self) -> Option<Cursor> {
	// 	self.inner.write().await.cursor.poll().await
	// }
}
