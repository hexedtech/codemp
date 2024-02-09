//! # MultiPlayer Code Editing lib
//!
//! ![just a nice pic](https://alemi.dev/img/about-slice-1.png)
//!
//! > the core library of the codemp project, driving all editor plugins
//!
//! ## structure
//! The main entrypoint is the [Client] object, that maintains a connection and can 
//! be used to join workspaces or attach to buffers. It contains the underlying [Workspace] and 
//! stores active controllers.
//! 
//! Some actions will return structs implementing the [api::Controller] trait. These can be polled 
//! for new stream events ([api::Controller::poll]/[api::Controller::recv]), which will be returned in order. 
//! Blocking and callback variants are also implemented. The [api::Controller] can also be used to send new 
//! events to the server ([api::Controller::send]).
//!
//! Each operation on a buffer is represented as an [woot::crdt::Op]. The underlying buffer is a
//! [WOOT CRDT](https://inria.hal.science/file/index/docid/71240/filename/RR-5580.pdf),
//! but to use this library it's only sufficient to know that all WOOT buffers that have received
//! the same operations converge to the same state, and that operations might not get integrated
//! immediately but instead deferred until compatible.
//!
//! ## features
//! * `api`    : include traits for core interfaces under [api] (default enabled)
//! * `woot`   : include the underlying CRDT library and re-exports it (default enabled)
//! * `proto`  : include GRCP protocol definitions under [proto] (default enabled)
//! * `client` : include the local [client] implementation (default enabled)
//! 
//! ## examples
//! most methods are split between the [Client] itself and the current [Workspace]
//!
//! ### async
//! ```rust,no_run
//! use codemp::api::{Controller, TextChange};
//!
//! # async fn async_example() -> codemp::Result<()> {
//! // creating a client session will immediately attempt to connect
//! let mut session = codemp::Client::new("http://alemi.dev:50053").await?;
//!
//! // login first, obtaining a new token granting access to 'some_workspace'
//! session.login(
//!   "username".to_string(),
//!   "password".to_string(),
//!   Some("some_workspace".to_string())
//! ).await?;
//! 
//! // join a remote workspace, obtaining a workspace handle
//! let workspace = session.join_workspace("some_workspace").await?;
//!
//! workspace.cursor().send(   // move cursor
//!   codemp::proto::cursor::CursorPosition {
//!     buffer: "test.txt".into(),
//!     start: codemp::proto::cursor::RowCol { row: 0, col: 0 },
//!     end: codemp::proto::cursor::RowCol { row: 0, col: 1 },
//!   }
//! )?;
//! let op = workspace.cursor().recv().await?; // receive event from server
//! println!("received cursor event: {:?}", op);
//! 
//! // attach to a new buffer and execute operations on it
//! workspace.create("test.txt").await?;   // create new buffer
//! let buffer = workspace.attach("test.txt").await?; // attach to it
//! let local_change = TextChange { span: 0..0, content: "hello!".into() };
//! buffer.send(local_change)?; // insert some text
//! let remote_change = buffer.recv().await?; // await remote change
//! #
//! # Ok(())
//! # }
//! ```
//!
//! it's always possible to get a [Workspace] reference using [Client::get_workspace]
//!
//! ### sync
//! if async is not viable, a solution might be keeping a global tokio runtime and blocking on it:
//!
//! ```rust,no_run
//! # use std::sync::Arc;
//! # use codemp::api::Controller;
//! #
//! # fn sync_example() -> codemp::Result<()> {
//! let rt = tokio::runtime::Runtime::new().unwrap();
//! let mut session = rt.block_on( // using block_on allows calling async code
//!   codemp::Client::new("http://alemi.dev:50051")
//! )?;
//!
//! rt.block_on(session.login(
//!   "username".to_string(),
//!   "password".to_string(),
//!   Some("some_workspace".to_string())
//! ))?;
//! 
//! let workspace = rt.block_on(session.join_workspace("some_workspace"))?;
//!
//! // attach to buffer and blockingly receive events
//! let buffer = rt.block_on(workspace.attach("test.txt"))?; // attach to buffer, must already exist
//! while let Ok(op) = rt.block_on(buffer.recv()) {   // must pass runtime
//!   println!("received buffer event: {:?}", op);
//! }
//! #
//! # Ok(())
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

/// workspace operations
#[cfg(feature = "client")]
pub mod workspace;

/// all-in-one imports : `use codemp::prelude::*;`
pub mod prelude;

/// underlying OperationalTransform library used, re-exported
#[cfg(feature = "woot")]
pub use woot;

/// protocol types and services auto-generated by grpc
#[cfg(feature = "proto")]
#[allow(non_snake_case)]
pub mod proto {
	pub mod common {
		tonic::include_proto!("common");

		impl From<uuid::Uuid> for Identity {
			fn from(id: uuid::Uuid) -> Self {
				Identity { id: id.to_string() }
			}
		}

		impl From<&uuid::Uuid> for Identity {
			fn from(id: &uuid::Uuid) -> Self {
				Identity { id: id.to_string() }
			}
		}
	
		impl From<Identity> for uuid::Uuid {
			fn from(value: Identity) -> Self {
				uuid::Uuid::parse_str(&value.id).expect("invalid uuid in identity")
			}
		}

		impl From<&Identity> for uuid::Uuid {
			fn from(value: &Identity) -> Self {
				uuid::Uuid::parse_str(&value.id).expect("invalid uuid in identity")
			}
		}
	}


	pub mod files {
		tonic::include_proto!("files");

		impl From<String> for BufferNode {
			fn from(value: String) -> Self {
				BufferNode { path: value }
			}
		}

		impl From<&str> for BufferNode {
			fn from(value: &str) -> Self {
				BufferNode { path: value.to_string() }
			}
		}

		impl From<BufferNode> for String {
			fn from(value: BufferNode) -> Self {
				value.path
			}
		}
	}

	pub mod buffer { tonic::include_proto!("buffer"); }
	pub mod cursor { tonic::include_proto!("cursor"); }
	pub mod workspace { tonic::include_proto!("workspace"); }
	pub mod auth { tonic::include_proto!("auth"); }
}

pub use errors::Error;
pub use errors::Result;

#[cfg(feature = "client")]
pub use client::Client;

#[cfg(feature = "client")]
pub use workspace::Workspace;


