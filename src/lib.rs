//! # Code MultiPlexer - cooperative development
//!
//! `codemp` is an async client library to create cooperation tools for any text editor.
//!
//! It is built as a batteries-included client library managing an authenticated user, multiple
//! workspaces each containing any number of buffers.
//!
//! The [`Client`] is completely managed by the library itself, making its use simple across async
//! contexts and FFI boundaries. All memory is managed by the library itself, which gives out always
//! atomic reference-counted pointers to internally mutable objects. Asynchronous actions are
//! abstracted away by the [`api::Controller`], providing an unopinionated approach with both
//! callback-based and blocking-based APIs.
//!
//! The library also provides ready-to-use bindings in a growing number of other programming languages,
//! to support a potentially infinite number of editors.
//!
//! # Overview
//! The main entrypoint is [`Client::connect`], which establishes an authenticated connection with
//! a supported remote server and returns a [`Client`] handle to interact with it.
//!
//! ```no_run
//! # async {
//! let client = codemp::Client::connect(
//!   codemp::api::Config::new(
//!     "mail@example.net",
//!     "dont-use-this-password"
//!   )
//! )
//!   .await
//!   .expect("failed to connect!");
//! # };
//! ```
//!
//! A [`Client`] can acquire a [`Workspace`] handle by joining an existing one it can access with
//! [`Client::join_workspace`] or create a new one with [`Client::create_workspace`].
//!
//! ```no_run
//! # async {
//! #  let client = codemp::Client::connect(codemp::api::Config::new("", "")).await.unwrap();
//! client.create_workspace("my-workspace").await.expect("failed to create workspace!");
//! let workspace = client.join_workspace("my-workspace").await.expect("failed to attach!");
//! # };
//! ```
//!
//! A [`Workspace`] handle can be used to acquire a [`cursor::Controller`] to track remote [`api::Cursor`]s
//! and one or more [`buffer::Controller`] to send and receive [`api::TextChange`]s.
//!
//! ```no_run
//! # async {
//! #  let client = codemp::Client::connect(codemp::api::Config::new("", "")).await.unwrap();
//! # client.create_workspace("").await.unwrap();
//! # let workspace = client.join_workspace("").await.unwrap();
//! use codemp::api::Controller; // needed to access trait methods 
//! let cursor = workspace.cursor();
//! let event = cursor.recv().await.expect("disconnected while waiting for event!");
//! println!("user {} moved on buffer {}", event.user.unwrap_or_default(), event.buffer);
//! # };
//! ```
//!
//! Internally, [`buffer::Controller`]s store the buffer state as a [diamond_types] CRDT, guaranteeing
//! eventual consistency. Each [`api::TextChange`] is translated in a network counterpart that is
//! guaranteed to converge.
//!
//! ```no_run
//! # async {
//! #  let client = codemp::Client::connect(codemp::api::Config::new("", "")).await.unwrap();
//! # client.create_workspace("").await.unwrap();
//! # let workspace = client.join_workspace("").await.unwrap();
//! # use codemp::api::Controller;
//! let buffer = workspace.attach("/some/file.txt").await.expect("failed to attach");
//! buffer.content(); // force-sync
//! if let Some(change) = buffer.try_recv().await.unwrap() {
//!   println!("content: {}, span: {}-{}", change.content, change.start, change.end);
//! } // if None, no changes are currently available
//! # };
//! ```
//!
//! ## FFI
//! As mentioned, we provide bindings in various programming languages. To obtain them, you can
//! compile with the appropriate feature flag. Currently, the following are supported:
//! * `lua`
//! * `javascript`
//! * `java` (requires additional build steps to be usable)
//! * `python`
//!
//! For some of these, ready-to-use packages are available in various registries:
//! * [PyPI (python)](https://pypi.org/project/codemp)
//! * [npm (javascript)](https://www.npmjs.com/package/codemp)
//!
#![doc(html_logo_url = "https://code.mp/static/logo-round.png")]
#![doc(html_favicon_url = "https://code.mp/static/favicon.ico")]

/// core structs and traits
pub mod api;

/// cursor related types and controller
pub mod cursor;

/// buffer related types and controller
pub mod buffer;

/// workspace handle and operations
pub mod workspace;
pub use workspace::Workspace;

/// client handle, containing all of the above
pub mod client;
pub use client::Client;

/// crate error types
pub mod errors;

/// all-in-one imports : `use codemp::prelude::*;`
pub mod prelude;

/// common utils used in this library and re-exposed
pub mod ext;

/// language-specific ffi "glue"
pub mod ffi;

/// internal network services and interceptors
pub(crate) mod network;

/// Get the current version of the client
pub fn version() -> String {
	env!("CARGO_PKG_VERSION").to_owned()
}
