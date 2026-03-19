//! ### Prelude
//! All-in-one renamed imports with `use codemp::prelude::*`.

pub use crate::api::{
	TextChange as CodempTextChange, Config as CodempConfig,
	AsyncReceiver as CodempAsyncReceiver, AsyncSender as CodempAsyncSender,
	Controller as CodempController,
	BufferUpdate as CodempBufferUpdate,
};

pub use crate::proto::{
	common::UserInfo as CodempUserInfo,
	files::BufferNode as CodempBufferNode, files::BufferAttributes as CodempBufferAttributes,
	buffer::BufferEvent as CodempBufferEvent,
	cursor::CursorEvent as CodempCursorEvent, cursor::CursorUpdate as CodempCursorUpdate,
	cursor::CursorPosition as CodempCursorPosition, cursor::RowCol as CodempRowCol,
	session::SessionEvent as CodempSessionEvent, session::WorkspaceIdentifier as CodempWorkspaceIdentifier,
	workspace::WorkspaceEvent as CodempWorkspaceEvent,
};

pub use crate::{
	buffer::Controller as CodempBufferController, client::Client as CodempClient,
	cursor::Controller as CodempCursorController, workspace::Workspace as CodempWorkspace,
};
