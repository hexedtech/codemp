use crate::prelude::*;
use mlua::prelude::*;
use mlua_codemp_patch as mlua;

use super::ext::a_sync::a_sync;
use super::ext::from_lua_serde;

impl LuaUserData for CodempWorkspace {
	fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| {
			Ok(format!("{:?}", this))
		});
		methods.add_method(
			"create",
			|_, this, (name,): (String,)| a_sync! { this => this.create(&name).await? },
		);

		methods.add_method(
			"attach",
			|_, this, (name,): (String,)| a_sync! { this => this.attach(&name).await? },
		);

		methods.add_method("detach", |_, this, (name,): (String,)| {
			Ok(this.detach(&name))
		});

		methods.add_method(
			"delete",
			|_, this, (name,): (String,)| a_sync! { this => this.delete(&name).await? },
		);

		methods.add_method("get_buffer", |_, this, (name,): (String,)| {
			Ok(this.buffer_by_name(&name))
		});

		methods.add_method(
			"fetch_buffers",
			|_, this, ()| a_sync! { this => this.fetch_buffers().await? },
		);
		methods.add_method(
			"fetch_users",
			|_, this, ()| a_sync! { this => this.fetch_users().await? },
		);

		methods.add_method(
			"filetree",
			|_, this, (filter, strict): (Option<String>, Option<bool>)| {
				Ok(this.filetree(filter.as_deref(), strict.unwrap_or(false)))
			},
		);

		methods.add_method("user_list", |_, this, ()| Ok(this.user_list()));

		methods.add_method("recv", |_, this, ()| a_sync! { this => this.recv().await? });

		methods.add_method(
			"try_recv",
			|_, this, ()| a_sync! { this => this.try_recv().await? },
		);

		methods.add_method("poll", |_, this, ()| a_sync! { this => this.poll().await? });

		methods.add_method("callback", |_, this, (cb,): (LuaFunction,)| {
			Ok(this.callback(move |controller: CodempWorkspace| {
				super::ext::callback().invoke(cb.clone(), controller)
			}))
		});

		methods.add_method("clear_callback", |_, this, ()| Ok(this.clear_callback()));
	}

	fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("name", |_, this| Ok(this.id()));
		fields.add_field_method_get("cursor", |_, this| Ok(this.cursor()));
		fields.add_field_method_get("active_buffers", |_, this| Ok(this.buffer_list()));
		// fields.add_field_method_get("users", |_, this| Ok(this.0.users())); // TODO
	}
}

from_lua_serde! { CodempEvent }
impl LuaUserData for CodempEvent {
	fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| {
			Ok(format!("{:?}", this))
		});
	}

	fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("type", |_, this| match this {
			CodempEvent::FileTreeUpdated(_) => Ok("filetree"),
			CodempEvent::UserJoin(_) => Ok("join"),
			CodempEvent::UserLeave(_) => Ok("leave"),
		});
		fields.add_field_method_get("value", |_, this| match this {
			CodempEvent::FileTreeUpdated(x)
			| CodempEvent::UserJoin(x)
			| CodempEvent::UserLeave(x) => Ok(x.clone()),
		});
	}
}
