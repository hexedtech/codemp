//! ### Prelude
//! 
//! all-in-one renamed imports with `use codemp::prelude::*`

pub use crate::{
	Error as CodempError,
	Result as CodempResult,
};

#[cfg(feature = "woot")]
pub use	crate::woot::crdt::Op as CodempOp;

#[cfg(feature = "api")]
pub use crate::api::{
	Controller as CodempController,
	TextChange as CodempTextChange,
};
	
#[cfg(feature = "client")]
pub use crate::{
	// Instance as CodempInstance,
	client::Client as CodempClient,
	workspace::Workspace as CodempWorkspace,
	workspace::UserInfo as CodempUserInfo,
	cursor::Controller as CodempCursorController,
	buffer::Controller as CodempBufferController,
};

#[cfg(feature = "proto")]
pub use crate::{
	proto::cursor::CursorPosition as CodempCursorPosition,
	proto::cursor::CursorEvent as CodempCursorEvent,
	proto::cursor::RowCol as CodempRowCol,
};
