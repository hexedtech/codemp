pub mod a_sync;
pub mod callback;
pub mod log;

pub(crate) use a_sync::tokio;
pub(crate) use callback::callback;

pub(crate) fn lua_parse_uuid(uuid: &str, pos: usize, name: &str) -> mlua::Result<uuid::Uuid> {
	use std::str::FromStr;
	match uuid::Uuid::from_str(uuid) {
		Ok(x) => Ok(x),
		Err(e) => Err(mlua::Error::BadArgument {
			pos,
			name: Some(name.to_string()),
			to: Some("Uuid::from_str".to_string()),
			cause: std::sync::Arc::new(mlua::Error::FromLuaConversionError {
				from: "string",
				to: "Uuid".to_string(),
				message: Some(e.to_string()),
			}),
		}),
	}
}

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

