use crate::ext::IgnorableError;
use crate::prelude::*;
use mlua::prelude::*;

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
	pub(crate) fn invoke(&self, key: String, arg: impl Into<CallbackArg>, cleanup: bool) {
		self.tx
			.send(LuaCallback::Invoke(key, arg.into(), cleanup))
			.unwrap_or_warn("error scheduling callback")
	}

	pub(crate) fn failure(&self, err: impl std::error::Error) {
		self.tx
			.send(LuaCallback::Fail(format!("callback returned error: {err:?}")))
			.unwrap_or_warn("error scheduling callback failure")
	}

	pub(crate) fn recv(&self, lua: &Lua) -> Option<(LuaFunction, CallbackArg)> {
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
				Ok(LuaCallback::Fail(msg)) => {
					tracing::error!("callback returned error: {msg}");
					None
				},
				Ok(LuaCallback::Invoke(key, arg, cleanup)) => {
					let cb = match lua.named_registry_value::<LuaFunction>(&key) {
						Ok(x) => x,
						Err(e) => {
							tracing::error!("could not get callback to invoke: {e}");
							return None;
						},
					};
					if cleanup {
						if let Err(e) = lua.unset_named_registry_value(&key) {
							tracing::warn!("could not unset callback from registry: {e}");
						}
					}
					Some((cb, arg))
				},
			},
		}
	}
}

pub(crate) enum LuaCallback {
	Fail(String),
	Invoke(String, CallbackArg, bool),
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
