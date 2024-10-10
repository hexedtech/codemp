use crate::prelude::*;
use mlua::prelude::*;
use mlua_codemp_patch as mlua;

use super::ext::a_sync::a_sync;
use super::ext::from_lua_serde;

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

		methods.add_method("clear_callback", |_, this, ()| Ok(this.clear_callback()));
		methods.add_method("callback", |_, this, (cb,): (LuaFunction,)|
			Ok(this.callback(move |controller: CodempCursorController|
				super::ext::callback().invoke(cb.clone(), controller)
			))
		);
	}
}

from_lua_serde! { CodempCursor }
impl LuaUserData for CodempCursor {
	fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("user", |_, this| Ok(this.user.clone()));
		fields.add_field_method_get("sel", |_, this| Ok(this.sel.clone()));
	}

	fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| {
			Ok(format!("{:?}", this))
		});
	}
}

from_lua_serde! { CodempSelection }
impl LuaUserData for CodempSelection {
	fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("buffer", |_, this| Ok(this.buffer.clone()));
		fields.add_field_method_get("start_row", |_, this| Ok(this.start_row));
		fields.add_field_method_get("start_col", |_, this| Ok(this.start_col));
		fields.add_field_method_get("end_row", |_, this| Ok(this.end_row));
		fields.add_field_method_get("end_col", |_, this| Ok(this.end_col));
	}

	fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| {
			Ok(format!("{:?}", this))
		});
	}
}
