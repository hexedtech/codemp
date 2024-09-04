//! # API
//! These traits and structs represent the main `codemp` library API.

/// a generic async provider for bidirectional communication
pub mod controller;

/// a generic representation of a text change
pub mod change;

/// representation for an user's cursor
pub mod cursor;

/// live events in workspaces
pub mod event;

/// data structure for remote users
pub mod user;

pub use controller::Controller;
pub use change::TextChange;
pub use cursor::Cursor;
pub use event::Event;
pub use user::User;
