use std::{collections::BTreeMap, sync::Arc};

use tokio::sync::RwLock;

use crate::{
	buffer::{controller::BufferController, handle::BufferHandle},
	errors::CodempError,
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
	buffers: RwLock<BTreeMap<Box<str>, Arc<BufferHandle>>>,
	// cursor: Arc<CursorTracker>,
	client: BufferController,
}

impl Workspace {
	pub async fn new(dest: &str) -> Result<Self, CodempError> {
		let client = BufferController::new(dest).await?;
		// let cursor = Arc::new(client.listen().await?);
		Ok(
			Workspace {
				buffers: RwLock::new(BTreeMap::new()),
				// cursor,
				client,
			}
		)
	}

	// Cursor
	// pub async fn cursor(&self) -> Arc<CursorTracker> {
	// 	self.cursor.clone()
	// }

	// Buffer
	pub async fn buffer(&self, path: &str) -> Option<Arc<BufferHandle>> {
		self.buffers.read().await.get(path).cloned()
	}

	pub async fn create(&self, path: &str, content: Option<&str>) -> Result<(), CodempError> {
		Ok(self.client.clone().create(path, content).await?)
	}

	pub async fn attach(&self, path: &str) -> Result<(), CodempError> {
		let controller = self.client.clone().attach(path).await?;
		self.buffers.write().await.insert(path.into(), Arc::new(controller));
		Ok(())
	}
}
