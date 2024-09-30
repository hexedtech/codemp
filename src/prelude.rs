//! ### Prelude
//! All-in-one renamed imports with `use codemp::prelude::*`.

pub use crate::api::{
	Config as CodempConfig, Controller as CodempController, Cursor as CodempCursor,
	Event as CodempEvent, TextChange as CodempTextChange, User as CodempUser,
};

pub use crate::{
	buffer::Controller as CodempBufferController, client::Client as CodempClient,
	cursor::Controller as CodempCursorController, workspace::Workspace as CodempWorkspace,
};
