//! # API
//! These traits and structs represent the main `codemp` library API.

/// codemp internal CRDT api, abstracting away its concepts
pub mod crdt;
pub use crdt::CRDT;

/// a generic async provider for bidirectional communication
pub mod controller;
pub use controller::{AsyncReceiver, AsyncSender, Controller};

/// a generic representation of a text change
pub mod change;

/// client configuration
pub mod config;

pub use change::{BufferUpdate, TextChange};
pub use config::Config;

