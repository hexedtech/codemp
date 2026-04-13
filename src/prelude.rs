//! ### Prelude
//! All-in-one renamed imports with `use codemp::prelude::*`.

pub use crate::api::{
	TextChange as CodempTextChange, Config as CodempConfig,
	AsyncReceiver as CodempAsyncReceiver, AsyncSender as CodempAsyncSender,
	Controller as CodempController,
	BufferUpdate as CodempBufferUpdate,
};

#[cfg(feature = "proto")]
pub use crate::proto::{
	common::UserInfo as CodempUserInfo,
	buffer::BufferNode as CodempBufferNode, buffer::BufferAttributes as CodempBufferAttributes,
	buffer::BufferEvent as CodempBufferEvent,
	cursor::CursorEvent as CodempCursorEvent, cursor::CursorUpdate as CodempCursorUpdate,
	cursor::CursorPosition as CodempCursorPosition, cursor::RowCol as CodempRowCol,
	session::SessionEvent as CodempSessionEvent, session::WorkspaceIdentifier as CodempWorkspaceIdentifier,
	workspace::WorkspaceEvent as CodempWorkspaceEvent,
};

#[cfg(feature = "client")]
pub use crate::{
	client::buffer::Controller as CodempBufferController, client::session::Session as CodempSession,
	client::cursor::Controller as CodempCursorController, client::workspace::Workspace as CodempWorkspace,
};
