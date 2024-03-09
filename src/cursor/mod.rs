//! ### cursor
//!
//! ![demo gif of early cursor sync in action](https://cdn.alemi.dev/codemp/demo-nvim.gif)
//! 
//! each user holds a cursor, which consists of multiple highlighted region 
//! on a specific buffer

pub(crate) mod worker;

/// cursor controller implementation
pub mod controller;

pub use controller::CursorController as Controller;
