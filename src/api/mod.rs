//! # api
//!
//! these traits represent the internal api for the codemp library.
//! more methods and structs are provided but these are the core interfaces to 
//! interact with the client.

/// a generic async provider for bidirectional communication
pub mod controller;

/// a generic representation of a text change
pub mod change;

/// representation for an user's cursor
pub mod cursor;

/// workspace events
pub mod event;

/// data structure for service users
pub mod user;

pub use controller::Controller;
pub use change::TextChange;
pub use cursor::Cursor;
pub use event::Event;
pub use user::User;
