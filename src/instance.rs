use std::sync::Arc;

use tokio::sync::Mutex;

use crate::{
	buffer::controller::BufferController,
	errors::CodempError, client::CodempClient, cursor::controller::CursorController,
};


use tokio::runtime::Runtime;

lazy_static::lazy_static! {
	static ref RUNTIME  : Runtime  = Runtime::new().expect("could not create tokio runtime");
	static ref INSTANCE : Instance = Instance::default();
}

pub struct Instance {
	client: Mutex<Option<CodempClient>>,
}

impl Default for Instance {
	fn default() -> Self {
		Instance { client: Mutex::new(None) }
	}
}

// TODO these methods repeat a lot of code but Mutex makes it hard to simplify

impl Instance {
	pub async fn connect(&self, addr: &str) -> Result<(), CodempError> {
		*self.client.lock().await = Some(CodempClient::new(addr).await?);
		Ok(())
	}

	pub async fn join(&self, session: &str) -> Result<(), CodempError> {
		self.client
			.lock()
			.await
			.as_mut()
			.ok_or(CodempError::InvalidState { msg: "connect first".into() })?
			.join(session)
			.await?;

		Ok(())
	}

	pub async fn create(&self, path: &str, content: Option<&str>) -> Result<(), CodempError> {
		self.client
			.lock()
			.await
			.as_mut()
			.ok_or(CodempError::InvalidState { msg: "connect first".into() })?
			.create(path, content)
			.await?;

		Ok(())
	}

	pub async fn get_cursor(&self) -> Result<Arc<CursorController>, CodempError> {
		self.client
			.lock()
			.await
			.as_mut()
			.ok_or(CodempError::InvalidState { msg: "connect first".into() })?
			.get_cursor()
			.ok_or(CodempError::InvalidState { msg: "join a workspace first".into() })
	}

	pub async fn get_buffer(&self, path: &str) -> Result<Arc<BufferController>, CodempError> {
		self.client
			.lock()
			.await
			.as_mut()
			.ok_or(CodempError::InvalidState { msg: "connect first".into() })?
			.get_buffer(path)
			.ok_or(CodempError::InvalidState { msg: "join a workspace or create requested buffer first".into() })
	}
}
