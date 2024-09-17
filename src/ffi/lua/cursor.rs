use mlua_codemp_patch as mlua;
use mlua::prelude::*;
use crate::prelude::*;

use super::ext::a_sync::a_sync;
use super::ext::from_lua_serde;
use super::ext::lua_tuple;

impl LuaUserData for CodempCursorController {
	fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(format!("{:?}", this)));

		methods.add_method("send", |_, this, (cursor,):(CodempCursor,)|
			a_sync! { this => this.send(cursor).await? }
		);
		methods.add_method("try_recv", |_, this, ()|
			a_sync! { this => this.try_recv().await? }
		);
		methods.add_method("recv", |_, this, ()| a_sync! { this => this.recv().await? });
		methods.add_method("poll", |_, this, ()| a_sync! { this => this.poll().await? });

		methods.add_method("stop", |_, this, ()| Ok(this.stop()));

		methods.add_method("clear_callback", |_, this, ()| { this.clear_callback(); Ok(()) });
		methods.add_method("callback", |_, this, (cb,):(LuaFunction,)| {
			this.callback(move |controller: CodempCursorController| super::ext::callback().invoke(cb.clone(), controller));
			Ok(())
		});
	}
}

from_lua_serde! { CodempCursor }
impl LuaUserData for CodempCursor {
	fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(format!("{:?}", this)));
	}

	fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("user", |_, this| Ok(this.user.clone()));
		fields.add_field_method_get("buffer", |_, this| Ok(this.buffer.clone()));
		fields.add_field_method_get("start",  |lua, this| lua_tuple(lua, this.start));
		fields.add_field_method_get("end", |lua, this| lua_tuple(lua, this.end));
		// add a 'finish' accessor too because in Lua 'end' is reserved
		fields.add_field_method_get("finish", |lua, this| lua_tuple(lua, this.end));
	}
}
