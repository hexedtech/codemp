//! # API
//! These traits and structs represent the main `codemp` library API.

/// a generic async provider for bidirectional communication
pub mod controller;

/// a generic representation of a text change
pub mod change;

/// client configuration
pub mod config;

pub use change::{BufferUpdate, TextChange};
pub use config::Config;
pub use controller::{AsyncReceiver, AsyncSender, Controller};
