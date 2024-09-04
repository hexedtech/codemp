//! ### Buffer
//! A buffer is a container of text, modifiable in sync by users.
//! It is built on top of [diamond_types] CRDT, guaranteeing that all peers which have received the
//! same set of operations will converge to the same content.

/// controller worker implementation
pub(crate) mod worker;

/// buffer controller implementation
pub mod controller;
pub use controller::BufferController as Controller;
