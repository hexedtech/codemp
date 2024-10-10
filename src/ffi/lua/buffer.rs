use crate::prelude::*;
use mlua::prelude::*;
use mlua_codemp_patch as mlua;

use super::ext::a_sync::a_sync;
use super::ext::from_lua_serde;

impl LuaUserData for CodempBufferController {
	fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| {
			Ok(format!("{:?}", this))
		});

		methods.add_method("send", |_, this, (change,): (CodempTextChange,)| {
			Ok(this.send(change)?)
		});

		methods.add_method(
			"try_recv",
			|_, this, ()| a_sync! { this => this.try_recv().await? },
		);
		methods.add_method("recv", |_, this, ()| a_sync! { this => this.recv().await? });
		methods.add_method("poll", |_, this, ()| a_sync! { this => this.poll().await? });
		methods.add_method_mut("ack", |_, this, (version,): (Vec<i64>,)| {
			Ok(this.ack(version))
		});

		methods.add_method(
			"content",
			|_, this, ()| a_sync! { this => this.content().await? },
		);

		methods.add_method("clear_callback", |_, this, ()| Ok(this.clear_callback()));
		methods.add_method("callback", |_, this, (cb,): (LuaFunction,)| {
			Ok(this.callback(move |controller: CodempBufferController| {
				super::ext::callback().invoke(cb.clone(), controller)
			}))
		});
	}
}

from_lua_serde! { CodempTextChange }
impl LuaUserData for CodempTextChange {
	fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("content", |_, this| Ok(this.content.clone()));
		fields.add_field_method_get("start", |_, this| Ok(this.start));
		fields.add_field_method_get("end", |_, this| Ok(this.end));
		// add a 'finish' accessor too because in Lua 'end' is reserved
		fields.add_field_method_get("finish", |_, this| Ok(this.end));
	}

	fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| {
			Ok(format!("{:?}", this))
		});
		methods.add_method("apply", |_, this, (txt,): (String,)| Ok(this.apply(&txt)));
	}
}

from_lua_serde! { CodempBufferUpdate }
impl LuaUserData for CodempBufferUpdate {
	fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("hash", |_, this| Ok(this.hash));
		fields.add_field_method_get("version", |_, this| Ok(this.version.clone()));
		fields.add_field_method_get("change", |_, this| Ok(this.change.clone()));
	}

	fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| {
			Ok(format!("{:?}", this))
		});
	}
}
