//! ### Prelude
//! 
//! all-in-one renamed imports with `use codemp::prelude::*`

pub use crate::{
	Error as CodempError,
	Result as CodempResult,
};

pub use crate::api::{
	Controller as CodempController,
	TextChange as CodempTextChange,
	Cursor as CodempCursor,
	Op as CodempOp,
};
	
pub use crate::{
	client::Client as CodempClient,
	workspace::Workspace as CodempWorkspace,
	workspace::UserInfo as CodempUserInfo,
	cursor::Controller as CodempCursorController,
	buffer::Controller as CodempBufferController,
};
