//! # MultiPlayer Code Editinglib
//!
//! ![just a nice pic](https://alemi.dev/img/about-slice-1.png)
//!
//! This is the core library of the codemp project.
//!
//! ## structure
//! The main entrypoint is the [Client] object, that maintains a connection and can 
//! be used to join workspaces or attach to buffers.
//! 
//! Some actions will return structs implementing the [Controller] trait. These can be polled 
//! for new stream events ([Controller::recv]), which will be returned in order. Blocking and 
//! callback variants are also implemented. The [Controller] can also be used to send new 
//! events to the server ([Controller::send]).
//!
//! Each operation on a buffer is represented as an [ot::OperationSeq].
//! A visualization about how OperationSeqs work is available
//! [here](http://operational-transformation.github.io/index.html),
//! but to use this library it's only sufficient to know that they can only 
//! be applied on buffers of some length and are transformable to be able to be 
//! applied in a different order while maintaining the same result.
//!
//! To generate Operation Sequences use helper methods from module [buffer::factory] (trait [buffer::OperationFactory]).
//!
//! ## features
//! * `proto` : include GRCP protocol definitions under [proto] (default enabled)
//! * `global`: provide a lazy_static global INSTANCE in [instance::global]
//! * `sync`  : wraps the [instance::a_sync::Instance] holder into a sync variant: [instance::sync::Instance]
//! 
//! ## examples
//! while the [Client] itself is the core structure implementing all methods, plugins will mostly
//! interact with [Instance] managers.
//!
//! ### async
//! this library is natively async and thus async usage should be preferred if possible with
//! [instance::a_sync::Instance]
//!
//! ```rust,no_run
//! use codemp::Controller;
//! use codemp::buffer::OperationFactory;
//!
//! # async fn async_example() -> codemp::Result<()> {
//! let session = codemp::Instance::default();   // create global session
//! session.connect("http://alemi.dev:50051").await?;   // connect to remote server
//! 
//! // join a remote workspace, obtaining a cursor controller
//! let cursor = session.join("some_workspace").await?;
//! cursor.send(   // move cursor
//!   codemp::proto::CursorPosition {
//!     buffer: "test.txt".into(),
//!     start: Some(codemp::proto::RowCol { row: 0, col: 0 }),
//!     end: Some(codemp::proto::RowCol { row: 0, col: 1 }),
//!   }
//! )?;
//! let op = cursor.recv().await?;   // listen for event
//! println!("received cursor event: {:?}", op);
//! 
//! // attach to a new buffer and execute operations on it
//! session.create("test.txt", None).await?;   // create new buffer
//! let buffer = session.attach("test.txt").await?; // attach to it
//! buffer.send(buffer.insert("hello", 0))?; // insert some text
//! if let Some(operation) = buffer.delta(4, "o world", 5) {
//!   buffer.send(operation)?; // replace with precision, if valid
//! }
//! assert_eq!(buffer.content(), "hello world");
//! #
//! # Ok(())
//! # }
//! ```
//!
//! ### sync
//! if async is not viable, including the feature `sync` will provide a sync-only [instance::sync::Instance] variant
//!
//! ```rust,no_run
//! # use codemp::instance::sync::Instance;
//! # use std::sync::Arc;
//! # use codemp::Controller;
//! #
//! # fn sync_example() -> codemp::Result<()> {
//! let session = Instance::default();   // instantiate sync variant
//! session.connect("http://alemi.dev:50051")?;   // connect to server
//!
//! // join remote workspace and handle cursor events with a callback
//! let cursor = session.join("some_workspace")?;   // join workspace
//! let (stop, stop_rx) = tokio::sync::mpsc::unbounded_channel();   // create stop channel
//! Arc::new(cursor).callback(   // register callback
//!   session.rt(), stop_rx,   // pass instance runtime and stop channel receiver
//!   | cursor_event | {  
//!     println!("received cursor event: {:?}", cursor_event);
//!   }
//! );
//!
//! // attach to buffer and blockingly receive events
//! let buffer = session.attach("test.txt")?; // attach to buffer, must already exist
//! while let Ok(op) = buffer.blocking_recv(session.rt()) {   // must pass runtime
//!   println!("received buffer event: {:?}", op);
//! }
//! #
//! # Ok(())
//! # }
//! ```
//!
//! ### global
//! if instantiating the [Instance] manager is not possible, adding the feature `global` will
//! provide a static lazyly-allocated global reference: [struct@instance::global::INSTANCE].
//!
//! ```rust,no_run
//! # use codemp::instance::sync::Instance;
//! # use std::sync::Arc;
//! use codemp::prelude::*;   // prelude includes everything with "Codemp" in front
//! # async fn global_example() -> codemp::Result<()> {
//! CODEMP_INSTANCE.connect("http://alemi.dev:50051").await?;   // connect to server
//! let cursor = CODEMP_INSTANCE.join("some_workspace").await?;   // join workspace
//! while let Ok(event) = cursor.recv().await {   // receive cursor events
//!   println!("received cursor event: {:?}", event);
//! }
//! #
//! #  Ok(())
//! # }
//! ```
//!
//! ## references
//!
//! ![another cool pic coz why not](https://alemi.dev/img/about-slice-2.png)
//!
//! check [codemp-vscode](https://github.com/codewithotherpeopleandchangenamelater/codemp-vscode)
//! or [codemp-nvim](https://github.com/codewithotherpeopleandchangenamelater/codemp-nvim)
//! or [codemp-server](https://github.com/codewithotherpeopleandchangenamelater/codemp-server) for
//! reference implementations.
//!
//! keep track of feature completedness with the 
//! [feature comparison matrix](https://github.com/orgs/codewithotherpeopleandchangenamelater/projects/3)
//!


/// cursor related types and controller
pub mod cursor;

/// buffer operations, factory, controller and types
pub mod buffer;

/// crate error types and helpers
pub mod errors;

/// underlying client session manager
pub mod client;

/// client wrapper to handle memory persistence
pub mod instance;

/// all-in-one imports : `use codemp::prelude::*;`
pub mod prelude;

/// underlying OperationalTransform library used, re-exported
pub use operational_transform as ot;

pub use client::Client;

#[cfg(feature = "sync")]      pub use instance::sync::Instance;
#[cfg(not(feature = "sync"))] pub use instance::a_sync::Instance;

/// protocol types and services auto-generated by grpc
#[cfg(feature = "proto")]
#[allow(non_snake_case)]
pub mod proto {
	tonic::include_proto!("codemp.buffer");
	tonic::include_proto!("codemp.cursor");
}

pub use errors::Error;
pub use errors::Result;

use std::sync::Arc;
use tokio::runtime::Runtime;

#[tonic::async_trait] // TODO move this somewhere?
pub(crate) trait ControllerWorker<T : Sized + Send + Sync> {
	type Controller : Controller<T>;
	type Tx;
	type Rx;

	fn subscribe(&self) -> Self::Controller;
	async fn work(self, tx: Self::Tx, rx: Self::Rx);
}

/// async and threadsafe handle to a generic bidirectional stream
///
/// this generic trait is implemented by actors managing stream procedures.
/// events can be enqueued for dispatching without blocking ([Controller::send]), and an async blocking 
/// api ([Controller::recv]) is provided to wait for server events. Additional sync blocking
/// ([Controller::blocking_recv]) and callback-based ([Controller::callback]) are implemented.
#[tonic::async_trait]
pub trait Controller<T : Sized + Send + Sync> : Sized + Send + Sync {
	/// type of upstream values, used in [Self::send]
	type Input;

	/// enqueue a new value to be sent
	fn send(&self, x: Self::Input) -> Result<()>;

	/// get next value from stream, blocking until one is available
	///
	/// this is just an async trait function wrapped by `async_trait`:
	///
	/// `async fn recv(&self) -> codemp::Result<T>;`
	async fn recv(&self) -> Result<T>;

	/// sync variant of [Self::recv], blocking invoking thread
	fn blocking_recv(&self, rt: &Runtime) -> Result<T> {
		rt.block_on(self.recv())
	}

	/// register a callback to be called for each received stream value
	///
	/// this will spawn a new task on given runtime invoking [Self::recv] in loop and calling given
	/// callback for each received value. a stop channel should be provided, and first value sent
	/// into it will stop the worker loop.
	///
	/// note: creating a callback handler will hold an Arc reference to the given controller,
	/// preventing it from being dropped (and likely disconnecting). using the stop channel is
	/// important for proper cleanup
	fn callback<F>(
		self: &Arc<Self>,
		rt: &tokio::runtime::Runtime,
		mut stop: tokio::sync::mpsc::UnboundedReceiver<()>,
		mut cb: F
	) where
		Self : 'static,
		F : FnMut(T) + Sync + Send + 'static
	{
		let _self = self.clone();
		rt.spawn(async move {
			loop {
				tokio::select! {
					Ok(data) = _self.recv() => cb(data),
					Some(()) = stop.recv() => break,
					else => break,
				}
			}
		});
	}
}
