//! ### Instance
//! 
//! This module provides convenience managers for the client instance
//!
//! the global instance reference is immutable and lazy-loaded, and requires `global` feature.

/// static global instance, allocated only if feature `global` is active
#[cfg(feature = "global")]
pub mod global {
	#[cfg(not(feature = "sync"))]
	lazy_static::lazy_static! {
		/// the global instance of codemp session
		pub static ref INSTANCE : super::a_sync::Instance = super::a_sync::Instance::default();
	}

	#[cfg(feature = "sync")]
	lazy_static::lazy_static! {
		/// the global instance of codemp session
		pub static ref INSTANCE : super::sync::Instance = super::sync::Instance::default();
	}
}

#[cfg(feature = "global")]
pub use global::INSTANCE;

/// async implementation of session instance
pub mod a_sync {
	use std::sync::Arc;
	
	use tokio::sync::Mutex;
	
	use crate::{
		buffer::controller::BufferController,
		errors::Error, client::Client, cursor::controller::CursorController,
	};

	/// persistant session manager for codemp client
	///
	/// will hold a tokio mutex over an optional client, and drop its reference when disconnecting.
	/// all methods are async because will await mutex availability
	#[derive(Default)]
	pub struct Instance {
		/// the tokio mutex containing a client, if connected
		client: Mutex<Option<Client>>,
	}
	
	impl Instance {
		/// connect to remote address instantiating a new client [crate::client::Client::new]
		pub async fn connect(&self, addr: &str) -> Result<(), Error> {
			*self.client.lock().await = Some(Client::new(addr).await?);
			Ok(())
		}
	
		/// threadsafe version of [crate::client::Client::join]
		pub async fn join(&self, session: &str) -> Result<Arc<CursorController>, Error> {
			self.client
				.lock().await
				.as_mut()
				.ok_or(Error::InvalidState { msg: "connect first".into() })?
				.join(session)
				.await
		}
	
		/// threadsafe version of [crate::client::Client::create]
		pub async fn create(&self, path: &str, content: Option<&str>) -> Result<(), Error> {
			self.client
				.lock().await
				.as_mut()
				.ok_or(Error::InvalidState { msg: "connect first".into() })?
				.create(path, content)
				.await
		}
	
		/// threadsafe version of [crate::client::Client::attach]
		pub async fn attach(&self, path: &str) -> Result<Arc<BufferController>, Error> {
			self.client
				.lock().await
				.as_mut()
				.ok_or(Error::InvalidState { msg: "connect first".into() })?
				.attach(path)
				.await
		}
	
		/// threadsafe version of [crate::client::Client::get_cursor]
		pub async fn get_cursor(&self) -> Result<Arc<CursorController>, Error> {
			self.client
				.lock().await
				.as_mut()
				.ok_or(Error::InvalidState { msg: "connect first".into() })?
				.get_cursor()
				.ok_or(Error::InvalidState { msg: "join workspace first".into() })
		}
	
		/// threadsafe version of [crate::client::Client::get_buffer]
		pub async fn get_buffer(&self, path: &str) -> Result<Arc<BufferController>, Error> {
			self.client
				.lock().await
				.as_mut()
				.ok_or(Error::InvalidState { msg: "connect first".into() })?
				.get_buffer(path)
				.ok_or(Error::InvalidState { msg: "join workspace first".into() })
		}
	
		/// threadsafe version of [crate::client::Client::leave_workspace]
		pub async fn leave_workspace(&self) -> Result<(), Error> {
			self.client
				.lock().await
				.as_mut()
				.ok_or(Error::InvalidState { msg: "connect first".into() })?
				.leave_workspace();
			Ok(())
		}
	
		/// threadsafe version of [crate::client::Client::disconnect_buffer]
		pub async fn disconnect_buffer(&self, path: &str) -> Result<bool, Error> {
			let res = self.client
				.lock().await
				.as_mut()
				.ok_or(Error::InvalidState { msg: "connect first".into() })?
				.disconnect_buffer(path);
			Ok(res)
		}
	}
}

/// sync implementation of session instance
pub mod sync {
	use std::sync::{Mutex, Arc};

	use tokio::runtime::{Runtime, Handle};

	use crate::{
		client::Client, Error,
		cursor::controller::CursorController,
		buffer::controller::BufferController
	};

	/// persistant session manager for codemp client
	///
	/// will hold a std mutex over an optional client, and drop its reference when disconnecting.
	/// also contains a tokio runtime to execute async futures on
	/// all methods are wrapped on a runtime.block_on and thus sync
	pub struct Instance {
		/// the std mutex containing a client, if connected
		client: Mutex<Option<Client>>,
		/// the tokio runtime
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
	
		/// return a reference to contained tokio runtime, to spawn tasks on
		pub fn rt(&self) -> &Handle { self.runtime.handle() }
	
		/// connect and store a client session, threadsafe and sync version of [crate::client::Client::new]
		pub fn connect(&self, addr: &str) -> Result<(), Error> {
			*self.client.lock().expect("client mutex poisoned") = Some(self.rt().block_on(Client::new(addr))?);
			Ok(())
		}
	
		/// threadsafe and sync version of [crate::client::Client::join]
		pub fn join(&self, session: &str) -> Result<Arc<CursorController>, Error> {
			self.if_client(|c| self.rt().block_on(c.join(session)))?
		}
	
		/// threadsafe and sync version of [crate::client::Client::create]
		pub fn create(&self, path: &str, content: Option<&str>) -> Result<(), Error> {
			self.if_client(|c| self.rt().block_on(c.create(path, content)))?
		}
	
		/// threadsafe and sync version of [crate::client::Client::attach]
		pub fn attach(&self, path: &str) -> Result<Arc<BufferController>, Error> {
			self.if_client(|c| self.rt().block_on(c.attach(path)))?
		}
	
		/// threadsafe and sync version of [crate::client::Client::get_cursor]
		pub fn get_cursor(&self) -> Result<Arc<CursorController>, Error> {
			self.if_client(|c| c.get_cursor().ok_or(Error::InvalidState { msg: "join workspace first".into() }))?
		}
	
		/// threadsafe and sync version of [crate::client::Client::get_buffer]
		pub fn get_buffer(&self, path: &str) -> Result<Arc<BufferController>, Error> {
			self.if_client(|c| c.get_buffer(path).ok_or(Error::InvalidState { msg: "join workspace or create requested buffer first".into() }))?
		}
	
		/// threadsafe and sync version of [crate::client::Client::leave_workspace]
		pub fn leave_workspace(&self) -> Result<(), Error> {
			self.if_client(|c| c.leave_workspace())
		}
	
		/// threadsafe and sync version of [crate::client::Client::disconnect_buffer]
		pub fn disconnect_buffer(&self, path: &str) -> Result<bool, Error> {
			self.if_client(|c| c.disconnect_buffer(path))
		}
	}
}
