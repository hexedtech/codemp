pub mod a_sync;
pub mod callback;
pub mod log;

use mlua_codemp_patch as mlua;
use mlua::prelude::*;

pub(crate) use a_sync::tokio;
pub(crate) use callback::callback;

pub(crate) fn lua_tuple<T: IntoLua>(lua: &Lua, (a, b): (T, T)) -> LuaResult<LuaTable> {
	let table = lua.create_table()?;
	table.set(1, a)?;
	table.set(2, b)?;
	Ok(table)
}

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
