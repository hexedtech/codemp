mod client;
mod workspace;
mod cursor;
mod buffer;
mod ext;

use mlua_codemp_patch as mlua;
use mlua::prelude::*;
use crate::prelude::*;

// define multiple entrypoints, so this library can have multiple names and still work
#[mlua::lua_module(name = "codemp")] fn entry_1(lua: &Lua) -> LuaResult<LuaTable> { entrypoint(lua) }
#[mlua::lua_module(name = "libcodemp")] fn entry_2(lua: &Lua) -> LuaResult<LuaTable> { entrypoint(lua) }
#[mlua::lua_module(name = "codemp_native")] fn entry_3(lua: &Lua) -> LuaResult<LuaTable> { entrypoint(lua) }
#[mlua::lua_module(name = "codemp_lua")] fn entry_4(lua: &Lua) -> LuaResult<LuaTable> { entrypoint(lua) }

fn entrypoint(lua: &Lua) -> LuaResult<LuaTable> {
	let exports = lua.create_table()?;

	// entrypoint
	exports.set("connect", lua.create_function(|_, (config,):(CodempConfig,)|
		ext::a_sync::a_sync! { => CodempClient::connect(config).await? }
	)?)?;

	// utils
	exports.set("hash", lua.create_function(|_, (txt,):(String,)|
		Ok(crate::ext::hash(txt))
	)?)?;

	// runtime
	exports.set("spawn_runtime_driver", lua.create_function(ext::a_sync::spawn_runtime_driver)?)?;
	exports.set("poll_callback", lua.create_function(|lua, ()| {
		// TODO pass args too
		let mut val = LuaMultiValue::new();
		if let Some((cb, arg)) = ext::callback::CHANNEL.recv() {
			val.push_back(LuaValue::Function(cb));
			val.push_back(arg.into_lua(lua)?);
		}
		Ok(val)
	})?)?;

	// logging
	exports.set("logger", lua.create_function(ext::log::logger)?)?;

	Ok(exports)
}

impl From::<crate::errors::ConnectionError> for LuaError {
	fn from(value: crate::errors::ConnectionError) -> Self {
		LuaError::runtime(value.to_string())
	}
}

impl From::<crate::errors::RemoteError> for LuaError {
	fn from(value: crate::errors::RemoteError) -> Self {
		LuaError::runtime(value.to_string())
	}
}

impl From::<crate::errors::ControllerError> for LuaError {
	fn from(value: crate::errors::ControllerError) -> Self {
		LuaError::runtime(value.to_string())
	}
}
