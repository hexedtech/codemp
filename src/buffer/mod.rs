//! ### Buffer
//! A buffer is a container of text, modifiable in sync by users.
//! It is built on top of [diamond_types] CRDT, guaranteeing that all peers which have received the
//! same set of operations will converge to the same content.

/// buffer controller implementation
pub mod controller;

pub(crate) mod worker;

pub use controller::BufferController as Controller;
