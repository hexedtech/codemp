//! ### Prelude
//! All-in-one renamed imports with `use codemp::prelude::*`.

pub use crate::api::{
	Controller as CodempController,
	TextChange as CodempTextChange,
	Cursor as CodempCursor,
	User as CodempUser,
	Event as CodempEvent,
};
	
pub use crate::{
	client::Client as CodempClient,
	workspace::Workspace as CodempWorkspace,
	cursor::Controller as CodempCursorController,
	buffer::Controller as CodempBufferController,
};
