pub mod a_sync;
pub mod callback;
pub mod log;

pub(crate) use a_sync::tokio;
pub(crate) use callback::callback;

macro_rules! from_lua_serde {
	($($t:ty)*) => {
		$(
			impl FromLua for $t {
				fn from_lua(value: LuaValue, lua: &Lua) -> LuaResult<$t> {
					lua.from_value(value)
				}
			}
		)*
	};
}

pub(crate) use from_lua_serde;
