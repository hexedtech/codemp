pub mod a_sync;
pub mod callback;
pub mod log;

pub(crate) use a_sync::tokio;
pub(crate) use callback::callback;

macro_rules! impl_lua_serde {
	($($t:ty)*) => {
		$(
			impl FromLua for $t {
				fn from_lua(value: LuaValue, lua: &Lua) -> LuaResult<$t> {
					lua.from_value(value)
				}
			}

			impl IntoLua for $t {
				fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
					lua.to_value(&self)
				}
			}
		)*
	};
}

pub(crate) use impl_lua_serde;
