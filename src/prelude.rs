//! ### Prelude
//! All-in-one renamed imports with `use codemp::prelude::*`.

pub use crate::api::{
	controller::AsyncReceiver as CodempAsyncReceiver, controller::AsyncSender as CodempAsyncSender,
	Config as CodempConfig, Controller as CodempController, Cursor as CodempCursor,
	Event as CodempEvent, TextChange as CodempTextChange, User as CodempUser,
	change::BufferUpdate as CodempBufferUpdate,
	cursor::Selection as CodempSelection,
};

pub use crate::{
	buffer::Controller as CodempBufferController, client::Client as CodempClient,
	cursor::Controller as CodempCursorController, workspace::Workspace as CodempWorkspace,
};
