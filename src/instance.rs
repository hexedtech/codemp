use std::sync::{Arc, Mutex};

use tokio::runtime::Runtime;

use crate::{
	buffer::controller::BufferController,
	errors::Error, client::Client, cursor::controller::CursorController,
};

#[cfg(feature = "global")]
pub mod global {
	lazy_static::lazy_static! {
		pub static ref INSTANCE : super::Instance = super::Instance::default();
	}
}

pub struct Instance {
	client: Mutex<Option<Client>>,
	runtime: Runtime,
}

impl Default for Instance {
	fn default() -> Self {
		Instance {
			client: Mutex::new(None),
			runtime: Runtime::new().expect("could not start tokio runtime"),
		}
	}
}

impl Instance {

	fn if_client<T>(&self, op: impl FnOnce(&mut Client) -> T) -> Result<T, Error> {
		if let Some(c) = self.client.lock().expect("client mutex poisoned").as_mut() {
			Ok(op(c))
		} else {
			Err(Error::InvalidState { msg: "connect first".into() })
		}
	}

	pub fn rt(&self) -> &Runtime { &self.runtime }

	pub fn connect(&self, addr: &str) -> Result<(), Error> {
		*self.client.lock().expect("client mutex poisoned") = Some(self.rt().block_on(Client::new(addr))?);
		Ok(())
	}

	pub fn join(&self, session: &str) -> Result<Arc<CursorController>, Error> {
		self.if_client(|c| self.rt().block_on(c.join(session)))?
	}

	pub fn create(&self, path: &str, content: Option<&str>) -> Result<(), Error> {
		self.if_client(|c| self.rt().block_on(c.create(path, content)))?
	}

	pub fn attach(&self, path: &str) -> Result<Arc<BufferController>, Error> {
		self.if_client(|c| self.rt().block_on(c.attach(path)))?
	}

	pub fn get_cursor(&self) -> Result<Arc<CursorController>, Error> {
		self.if_client(|c| c.get_cursor().ok_or(Error::InvalidState { msg: "join workspace first".into() }))?
	}

	pub fn get_buffer(&self, path: &str) -> Result<Arc<BufferController>, Error> {
		self.if_client(|c| c.get_buffer(path).ok_or(Error::InvalidState { msg: "join workspace or create requested buffer first".into() }))?
	}

	pub fn leave_workspace(&self) -> Result<(), Error> {
		self.if_client(|c| c.leave_workspace())
	}

	pub fn disconnect_buffer(&self, path: &str) -> Result<bool, Error> {
		self.if_client(|c| c.disconnect_buffer(path))
	}
}
