use crate::ext::IgnorableError;
use crate::prelude::*;
use mlua::prelude::*;
use mlua_codemp_patch as mlua;

pub(crate) fn callback() -> &'static CallbackChannel<LuaCallback> {
	static CHANNEL: std::sync::OnceLock<CallbackChannel<LuaCallback>> = std::sync::OnceLock::new();
	CHANNEL.get_or_init(CallbackChannel::default)
}

pub(crate) struct CallbackChannel<T> {
	tx: std::sync::Arc<tokio::sync::mpsc::UnboundedSender<T>>,
	rx: std::sync::Mutex<tokio::sync::mpsc::UnboundedReceiver<T>>,
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
		self.tx
			.send(LuaCallback::Invoke(cb, arg.into()))
			.unwrap_or_warn("error scheduling callback")
	}

	pub(crate) fn failure(&self, err: impl std::error::Error) {
		self.tx
			.send(LuaCallback::Fail(format!(
				"promise failed with error: {err:?}"
			)))
			.unwrap_or_warn("error scheduling callback failure")
	}

	pub(crate) fn recv(&self) -> Option<LuaCallback> {
		match self.rx.try_lock() {
			Err(e) => {
				tracing::debug!("backing off from callback mutex: {e}");
				None
			}
			Ok(mut lock) => match lock.try_recv() {
				Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
					tracing::error!("callback channel closed");
					None
				}
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

macro_rules! callback_args {
	($($name:ident : $t:ty ,)*) => {
		pub(crate) enum CallbackArg {
			Nil,
			$(
				$name($t),
			)*
		}

		impl IntoLua for CallbackArg {
			fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
				match self {
					Self::Nil => Ok(LuaValue::Nil),
					$(
						Self::$name(x) => x.into_lua(lua),
					)*
				}
			}
		}

		impl From<()> for CallbackArg {
			fn from(_value: ()) -> Self {
				Self::Nil
			}
		}

		$(
			impl From<$t> for CallbackArg {
				fn from(value: $t) -> Self {
					Self::$name(value)
				}
			}
		)*
	};
}

callback_args! {
	Str: String,
	VecStr: Vec<String>,
	VecUser: Vec<CodempUser>,
	Client: CodempClient,
	CursorController: CodempCursorController,
	BufferController: CodempBufferController,
	Workspace: CodempWorkspace,
	Event: CodempEvent,
	MaybeEvent: Option<CodempEvent>,
	Cursor: CodempCursor,
	MaybeCursor: Option<CodempCursor>,
	Selection: CodempSelection,
	MaybeSelection: Option<CodempSelection>,
	TextChange: CodempTextChange,
	MaybeTextChange: Option<CodempTextChange>,
	BufferUpdate: CodempBufferUpdate,
	MaybeBufferUpdate: Option<CodempBufferUpdate>,
}
