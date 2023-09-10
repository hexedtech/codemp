//! # api
//!
//! these traits represent the internal api for the codemp library.
//! more methods and structs are provided but these are the core interfaces to 
//! interact with the client.

/// a generic async provider for bidirectional communication
pub mod controller;

/// a helper trait to generate operation sequences
pub mod factory;

pub use controller::Controller;
pub use factory::OperationFactory;
