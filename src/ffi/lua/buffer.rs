use crate::prelude::*;
use mlua::prelude::*;

use super::ext::a_sync::a_sync;

super::ext::impl_lua_serde! { CodempTextChange CodempBufferUpdate }

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

		methods.add_method("clear_callback", move |lua, this, ()| {
			this.clear_callback();
			lua.unset_named_registry_value(&this.lua_callback_id())
		});

		methods.add_method("callback", move |lua, this, (cb,): (LuaFunction,)| {
			let key = this.lua_callback_id();
			lua.set_named_registry_value(&key, cb)?;
			Ok(this.callback(move |controller: CodempBufferController| {
				super::ext::callback().invoke(key.clone(), controller, false)
			}))
		});
	}
}

impl CodempBufferController {
	fn lua_callback_id(&self) -> String {
		format!("codemp-buffercontroller({}:{})-callback-registry", self.workspace_id(), self.path())
	}
}
