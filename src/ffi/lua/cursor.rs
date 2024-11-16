use crate::prelude::*;
use mlua::prelude::*;

use super::ext::a_sync::a_sync;

super::ext::impl_lua_serde! { CodempCursor CodempSelection }

impl LuaUserData for CodempCursorController {
	fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| {
			Ok(format!("{:?}", this))
		});

		methods.add_method("send", |_, this, (cursor,): (CodempSelection,)| {
			Ok(this.send(cursor)?)
		});
		methods.add_method(
			"try_recv",
			|_, this, ()| a_sync! { this => this.try_recv().await? },
		);
		methods.add_method("recv", |_, this, ()| a_sync! { this => this.recv().await? });
		methods.add_method("poll", |_, this, ()| a_sync! { this => this.poll().await? });

		methods.add_method("clear_callback", |lua, this, ()| {
			this.clear_callback();
			lua.unset_named_registry_value(&this.lua_callback_id())
		});
		methods.add_method("callback", |lua, this, (cb,): (LuaFunction,)| {
			let key = this.lua_callback_id();
			lua.set_named_registry_value(&key, cb)?;
			Ok(this.callback(move |controller: CodempCursorController| {
				super::ext::callback().invoke(key.clone(), controller, false)
			}))
		});
	}
}

impl CodempCursorController {
	fn lua_callback_id(&self) -> String {
		format!("codemp-cursorcontroller({})-callback-registry", self.workspace_id())
	}
}
