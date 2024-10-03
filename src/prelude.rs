//! ### Prelude
//! All-in-one renamed imports with `use codemp::prelude::*`.

pub use crate::api::{
	Controller as CodempController,
	controller::AsyncSender as CodempAsyncSender,
	controller::AsyncReceiver as CodempAsyncReceiver,
	TextChange as CodempTextChange,
	Cursor as CodempCursor,
	User as CodempUser,
	Event as CodempEvent,
	Config as CodempConfig,
};

pub use crate::{
	buffer::Controller as CodempBufferController, client::Client as CodempClient,
	cursor::Controller as CodempCursorController, workspace::Workspace as CodempWorkspace,
};
