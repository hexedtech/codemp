use mlua_codemp_patch as mlua;
use mlua::prelude::*;
use crate::prelude::*;
use crate::ext::IgnorableError;

lazy_static::lazy_static! {
	pub(crate) static ref CHANNEL: CallbackChannel = CallbackChannel::default();
}

pub(crate) struct CallbackChannel {
	tx: std::sync::Arc<tokio::sync::mpsc::UnboundedSender<(LuaFunction, CallbackArg)>>,
	rx: std::sync::Mutex<tokio::sync::mpsc::UnboundedReceiver<(LuaFunction, CallbackArg)>>
}

impl Default for CallbackChannel {
	fn default() -> Self {
		let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
		let rx = std::sync::Mutex::new(rx);
		Self {
			tx: std::sync::Arc::new(tx),
			rx,
		}
	}
}

impl CallbackChannel {
	pub(crate) fn send(&self, cb: LuaFunction, arg: impl Into<CallbackArg>) {
		self.tx.send((cb, arg.into()))
			.unwrap_or_warn("error scheduling callback")
	}

	pub(crate) fn recv(&self) -> Option<(LuaFunction, CallbackArg)> {
		match self.rx.try_lock() {
			Err(e) => {
				tracing::warn!("could not acquire callback channel mutex: {e}");
				None
			},
			Ok(mut lock) => match lock.try_recv() {
				Err(tokio::sync::mpsc::error::TryRecvError::Empty) => None,
				Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
					tracing::error!("callback channel closed");
					None
				},
				Ok((cb, arg)) => Some((cb, arg)),
			},
		}
	}
}

pub(crate) enum CallbackArg {
	Nil,
	Str(String),
	VecStr(Vec<String>),
	Client(CodempClient),
	CursorController(CodempCursorController),
	BufferController(CodempBufferController),
	Workspace(CodempWorkspace),
	Event(CodempEvent),
	Cursor(CodempCursor),
	MaybeCursor(Option<CodempCursor>),
	TextChange(CodempTextChange),
	MaybeTextChange(Option<CodempTextChange>),
}

impl IntoLua for CallbackArg {
	// TODO this basically calls .into_lua() on all enum variants
	//      i wish i could do this with a Box<dyn IntoLua> or an impl IntoLua
	//      but IntoLua requires Sized so it can't be made into an object
	fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
		match self {
			CallbackArg::Nil => Ok(LuaValue::Nil),
			CallbackArg::Str(x) => x.into_lua(lua),
			CallbackArg::Client(x) => x.into_lua(lua),
			CallbackArg::CursorController(x) => x.into_lua(lua),
			CallbackArg::BufferController(x) => x.into_lua(lua),
			CallbackArg::Workspace(x) => x.into_lua(lua),
			CallbackArg::VecStr(x) => x.into_lua(lua),
			CallbackArg::Event(x) => x.into_lua(lua),
			CallbackArg::Cursor(x) => x.into_lua(lua),
			CallbackArg::MaybeCursor(x) => x.into_lua(lua),
			CallbackArg::TextChange(x) => x.into_lua(lua),
			CallbackArg::MaybeTextChange(x) => x.into_lua(lua),
		}
	}
}

impl From<()> for CallbackArg { fn from(_: ()) -> Self { CallbackArg::Nil } }
impl From<String> for CallbackArg { fn from(value: String) -> Self { CallbackArg::Str(value) } }
impl From<CodempClient> for CallbackArg { fn from(value: CodempClient) -> Self { CallbackArg::Client(value) } }
impl From<CodempCursorController> for CallbackArg { fn from(value: CodempCursorController) -> Self { CallbackArg::CursorController(value) } }
impl From<CodempBufferController> for CallbackArg { fn from(value: CodempBufferController) -> Self { CallbackArg::BufferController(value) } }
impl From<CodempWorkspace> for CallbackArg { fn from(value: CodempWorkspace) -> Self { CallbackArg::Workspace(value) } }
impl From<Vec<String>> for CallbackArg { fn from(value: Vec<String>) -> Self { CallbackArg::VecStr(value) } }
impl From<CodempEvent> for CallbackArg { fn from(value: CodempEvent) -> Self { CallbackArg::Event(value) } }
impl From<CodempCursor> for CallbackArg { fn from(value: CodempCursor) -> Self { CallbackArg::Cursor(value) } }
impl From<Option<CodempCursor>> for CallbackArg { fn from(value: Option<CodempCursor>) -> Self { CallbackArg::MaybeCursor(value) } }
impl From<CodempTextChange> for CallbackArg { fn from(value: CodempTextChange) -> Self { CallbackArg::TextChange(value) } }
impl From<Option<CodempTextChange>> for CallbackArg { fn from(value: Option<CodempTextChange>) -> Self { CallbackArg::MaybeTextChange(value) } }
