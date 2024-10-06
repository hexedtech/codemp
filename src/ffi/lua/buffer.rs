use crate::buffer::controller::Delta;
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
			this.send(change)?;
			Ok(())
		});

		methods.add_method(
			"try_recv",
			|_, this, ()| a_sync! { this => this.try_recv().await? },
		);
		methods.add_method("recv", |_, this, ()| a_sync! { this => this.recv().await? });
		methods.add_method("poll", |_, this, ()| a_sync! { this => this.poll().await? });

		methods.add_method(
			"content",
			|_, this, ()| a_sync! { this => this.content().await? },
		);

		methods.add_method("clear_callback", |_, this, ()| {
			this.clear_callback();
			Ok(())
		});
		methods.add_method("callback", |_, this, (cb,): (LuaFunction,)| {
			this.callback(move |controller: CodempBufferController| {
				super::ext::callback().invoke(cb.clone(), controller)
			});
			Ok(())
		});
	}
}

from_lua_serde! { CodempTextChange }
impl LuaUserData for CodempTextChange {
	fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("content", |_, this| Ok(this.content.clone()));
		fields.add_field_method_get("start", |_, this| Ok(this.start));
		fields.add_field_method_get("end", |_, this| Ok(this.end));
		fields.add_field_method_get("hash", |_, this| Ok(this.hash));
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

impl LuaUserData for Delta {
	fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("change", |_, this| Ok(this.change.clone()));
	}

	fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
		methods.add_method_mut("ack", |_, this, ()| Ok(this.ack()));
	}
}
