//! ### Prelude
//! All-in-one renamed imports with `use codemp::prelude::*`.

pub use crate::api::{
	TextChange as CodempTextChange, Config as CodempConfig,
	AsyncReceiver as CodempAsyncReceiver, AsyncSender as CodempAsyncSender,
	Controller as CodempController,
};

pub use crate::proto::{
	files::BufferNode as CodempBufferNode, buffer::BufferEvent as CodempBufferEvent,
	cursor::CursorEvent as CodempCursorEvent, cursor::CursorUpdate as CodempCursorUpdate,
	cursor::CursorPosition as CodempCursorPosition, cursor::RowCol as CodempRowCol,
	session::SessionEvent as CodempSessionEvent, workspace::WorkspaceEvent as CodempWorkspaceEvent,
	session::WorkspaceIdentifier as CodempWorkspaceIdentifier, common::UserInfo as CodempUserInfo,
};

pub use crate::{
	buffer::Controller as CodempBufferController, client::Client as CodempClient,
	cursor::Controller as CodempCursorController, workspace::Workspace as CodempWorkspace,
};
