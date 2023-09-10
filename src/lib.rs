//! # MultiPlayer Code Editing lib
//!
//! ![just a nice pic](https://alemi.dev/img/about-slice-1.png)
//!
//! This is the core library of the codemp project.
//!
//! ## structure
//! The main entrypoint is the [Instance] object, that maintains a connection and can 
//! be used to join workspaces or attach to buffers. It contains the underlying [client::Client] and 
//! stores active controllers.
//! 
//! Some actions will return structs implementing the [api::Controller] trait. These can be polled 
//! for new stream events ([api::Controller::poll]/[api::Controller::recv]), which will be returned in order. 
//! Blocking and callback variants are also implemented. The [api::Controller] can also be used to send new 
//! events to the server ([api::Controller::send]).
//!
//! Each operation on a buffer is represented as an [ot::OperationSeq].
//! A visualization about how OperationSeqs work is available
//! [here](http://operational-transformation.github.io/index.html),
//! but to use this library it's only sufficient to know that they can only 
//! be applied on buffers of some length and are transformable to be able to be 
//! applied in a different order while maintaining the same result.
//!
//! To generate Operation Sequences use helper methods from module [api::factory] (trait [api::OperationFactory]).
//!
//! ## features
//! * `ot`    : include the underlying operational transform library (default enabled)
//! * `api`   : include traits for core interfaces under [api] (default enabled)
//! * `proto` : include GRCP protocol definitions under [proto] (default enabled)
//! * `client`: include the local [client] implementation (default enabled)
//! * `global`: provide a lazy_static global INSTANCE in [instance::global]
//! * `sync`  : wraps the [instance::a_sync::Instance] holder into a sync variant: [instance::sync::Instance]
//! 
//! ## examples
//! while the [client::Client] itself is the core structure implementing all methods, plugins will mostly
//! interact with [Instance] managers.
//!
//! ### async
//! this library is natively async and thus async usage should be preferred if possible with
//! [instance::a_sync::Instance]
//!
//! ```rust,no_run
//! use codemp::api::Controller;
//! use codemp::buffer::OperationFactory;
//! # use codemp::instance::a_sync::Instance;
//!
//! # async fn async_example() -> codemp::Result<()> {
//! let session = Instance::default();   // create global session
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
//! # use codemp::api::Controller;
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
//! # fn global_example() -> codemp::Result<()> {
//! CODEMP_INSTANCE.connect("http://alemi.dev:50051")?;   // connect to server
//! let cursor = CODEMP_INSTANCE.join("some_workspace")?;   // join workspace
//! std::thread::spawn(move || {
//!   loop {
//!     match cursor.try_recv() {
//!       Ok(Some(event)) => println!("received cursor event: {:?}", event),  // there might be more
//!       Ok(None) => std::thread::sleep(std::time::Duration::from_millis(10)),  // wait for more
//!       Err(e) => break,  // channel closed
//!     }
//!   }
//! });
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

#![doc(html_no_source)]


/// public traits exposed to clients
#[cfg(feature = "api")]
pub mod api;

/// cursor related types and controller
#[cfg(feature = "client")]
pub mod cursor;

/// buffer operations, factory, controller and types
#[cfg(feature = "client")]
pub mod buffer;

/// crate error types and helpers
pub mod errors;

/// underlying client session manager
#[cfg(feature = "client")]
pub mod client;

/// client wrapper to handle memory persistence
#[cfg(feature = "client")]
pub mod instance;

/// all-in-one imports : `use codemp::prelude::*;`
pub mod prelude;

/// underlying OperationalTransform library used, re-exported
#[cfg(feature = "ot")]
pub use operational_transform as ot;

/// protocol types and services auto-generated by grpc
#[cfg(feature = "proto")]
#[allow(non_snake_case)]
pub mod proto {
	tonic::include_proto!("codemp.buffer");
	tonic::include_proto!("codemp.cursor");
}


pub use errors::Error;
pub use errors::Result;

#[cfg(all(feature = "client", feature = "sync"))]
pub use instance::sync::Instance;

#[cfg(all(feature = "client", not(feature = "sync")))]
pub use instance::a_sync::Instance;

