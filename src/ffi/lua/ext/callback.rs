use mlua_codemp_patch as mlua;
use mlua::prelude::*;
use crate::prelude::*;
use crate::ext::IgnorableError;

pub(crate) fn callback() -> &'static CallbackChannel<LuaCallback> {
	static CHANNEL: std::sync::OnceLock<CallbackChannel<LuaCallback>> = std::sync::OnceLock::new();
	CHANNEL.get_or_init(CallbackChannel::default)
}

pub(crate) struct CallbackChannel<T> {
	tx: std::sync::Arc<tokio::sync::mpsc::UnboundedSender<T>>,
	rx: std::sync::Mutex<tokio::sync::mpsc::UnboundedReceiver<T>>
}

impl Default for CallbackChannel<LuaCallback> {
	fn default() -> Self {
		let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
		let rx = std::sync::Mutex::new(rx);
		Self {
			tx: std::sync::Arc::new(tx),
			rx,
		}
	}
}

impl CallbackChannel<LuaCallback> {
	pub(crate) fn invoke(&self, cb: LuaFunction, arg: impl Into<CallbackArg>) {
		self.tx.send(LuaCallback::Invoke(cb, arg.into()))
			.unwrap_or_warn("error scheduling callback")
	}

	pub(crate) fn failure(&self, err: impl std::error::Error) {
		self.tx.send(LuaCallback::Fail(format!("promise failed with error: {err:?}")))
			.unwrap_or_warn("error scheduling callback failure")
	}

	pub(crate) fn recv(&self) -> Option<LuaCallback> {
		match self.rx.try_lock() {
			Err(e) => {
				tracing::debug!("backing off from callback mutex: {e}");
				None
			},
			Ok(mut lock) => match lock.try_recv() {
				Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
					tracing::error!("callback channel closed");
					None
				},
				Err(tokio::sync::mpsc::error::TryRecvError::Empty) => None,
				Ok(cb) => Some(cb),
			},
		}
	}
}

pub(crate) enum LuaCallback {
	Fail(String),
	Invoke(LuaFunction, CallbackArg),
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
