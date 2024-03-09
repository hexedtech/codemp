//! ### Prelude
//! 
//! all-in-one renamed imports with `use codemp::prelude::*`

pub use crate::{
	Error as CodempError,
	Result as CodempResult,
};

pub use	crate::woot::crdt::Op as CodempOp;

pub use crate::api::{
	Controller as CodempController,
	TextChange as CodempTextChange,
};
	
pub use crate::{
	// Instance as CodempInstance,
	client::Client as CodempClient,
	workspace::Workspace as CodempWorkspace,
	workspace::UserInfo as CodempUserInfo,
	cursor::Controller as CodempCursorController,
	buffer::Controller as CodempBufferController,
};
