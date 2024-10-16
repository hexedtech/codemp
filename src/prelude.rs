//! ### Prelude
//! All-in-one renamed imports with `use codemp::prelude::*`.

pub use crate::api::{
	AsyncReceiver as CodempAsyncReceiver, AsyncSender as CodempAsyncSender,
	BufferUpdate as CodempBufferUpdate, Config as CodempConfig, Controller as CodempController,
	Cursor as CodempCursor, Event as CodempEvent, Selection as CodempSelection,
	TextChange as CodempTextChange, User as CodempUser,
};

pub use crate::{
	buffer::Controller as CodempBufferController, client::Client as CodempClient,
	cursor::Controller as CodempCursorController, workspace::Workspace as CodempWorkspace,
};
