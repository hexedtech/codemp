//! # api
//!
//! these traits represent the internal api for the codemp library.
//! more methods and structs are provided but these are the core interfaces to 
//! interact with the client.

/// a generic async provider for bidirectional communication
pub mod controller;

/// a generic representation of a text change
pub mod change;

pub use controller::Controller;
pub use change::TextChange;
