use std::sync::Arc;

use tokio::sync::Mutex;

use crate::{
	buffer::controller::BufferController,
	errors::Error, client::Client, cursor::controller::CursorController,
};


use tokio::runtime::Runtime;

lazy_static::lazy_static! {
	pub static ref RUNTIME  : Runtime  = Runtime::new().expect("could not create tokio runtime");
	pub static ref INSTANCE : Instance = Instance::default();
}

pub struct Instance {
	client: Mutex<Option<Client>>,
}

impl Default for Instance {
	fn default() -> Self {
		Instance { client: Mutex::new(None) }
	}
}

// TODO these methods repeat a lot of code but Mutex makes it hard to simplify

impl Instance {
	pub async fn connect(&self, addr: &str) -> Result<(), Error> {
		*self.client.lock().await = Some(Client::new(addr).await?);
		Ok(())
	}

	pub async fn join(&self, session: &str) -> Result<Arc<CursorController>, Error> {
		self.client
			.lock()
			.await
			.as_mut()
			.ok_or(Error::InvalidState { msg: "connect first".into() })?
			.join(session)
			.await
	}

	pub async fn create(&self, path: &str, content: Option<&str>) -> Result<(), Error> {
		self.client
			.lock()
			.await
			.as_mut()
			.ok_or(Error::InvalidState { msg: "connect first".into() })?
			.create(path, content)
			.await
	}

	pub async fn attach(&self, path: &str) -> Result<Arc<BufferController>, Error> {
		self.client
			.lock()
			.await
			.as_mut()
			.ok_or(Error::InvalidState { msg: "connect first".into() })?
			.attach(path)
			.await
	}

	pub async fn get_cursor(&self) -> Result<Arc<CursorController>, Error> {
		self.client
			.lock()
			.await
			.as_mut()
			.ok_or(Error::InvalidState { msg: "connect first".into() })?
			.get_cursor()
			.ok_or(Error::InvalidState { msg: "join a workspace first".into() })
	}

	pub async fn get_buffer(&self, path: &str) -> Result<Arc<BufferController>, Error> {
		self.client
			.lock()
			.await
			.as_mut()
			.ok_or(Error::InvalidState { msg: "connect first".into() })?
			.get_buffer(path)
			.ok_or(Error::InvalidState { msg: "join a workspace or create requested buffer first".into() })
	}

	pub async fn leave_workspace(&self) -> Result<(), Error> {
		self.client
			.lock()
			.await
			.as_mut()
			.ok_or(Error::InvalidState { msg: "connect first".into() })?
			.leave_workspace();
		Ok(())
	}

	pub async fn disconnect_buffer(&self, path: &str) -> Result<bool, Error> {
		Ok(
			self.client
				.lock()
				.await
				.as_mut()
				.ok_or(Error::InvalidState { msg: "connect first".into() })?
				.disconnect_buffer(path)
		)
	}
}
