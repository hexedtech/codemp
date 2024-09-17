use mlua_codemp_patch as mlua;
use mlua::prelude::*;
use crate::prelude::*;
use crate::workspace::DetachResult;

use super::ext::a_sync::a_sync;
use super::ext::from_lua_serde;

impl LuaUserData for CodempWorkspace {
	fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(format!("{:?}", this)));
		methods.add_method("create", |_, this, (name,):(String,)|
			a_sync! { this => this.create(&name).await? }
		);

		methods.add_method("attach", |_, this, (name,):(String,)|
			a_sync! { this => this.attach(&name).await? }
		);

		methods.add_method("detach", |_, this, (name,):(String,)|
			Ok(matches!(this.detach(&name), DetachResult::Detaching | DetachResult::AlreadyDetached))
		);

		methods.add_method("delete", |_, this, (name,):(String,)|
			a_sync! { this => this.delete(&name).await? }
		);

		methods.add_method("get_buffer", |_, this, (name,):(String,)| Ok(this.buffer_by_name(&name)));

		methods.add_method("event", |_, this, ()|
			a_sync! { this => this.event().await? }
		);

		methods.add_method("fetch_buffers", |_, this, ()|
			a_sync! { this => this.fetch_buffers().await? }
		);
		methods.add_method("fetch_users", |_, this, ()|
			a_sync! { this => this.fetch_users().await? }
		);

		methods.add_method("filetree", |_, this, (filter, strict,):(Option<String>, Option<bool>,)|
			Ok(this.filetree(filter.as_deref(), strict.unwrap_or(false)))
		);
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
		methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(format!("{:?}", this)));
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
				| CodempEvent::UserLeave(x)
				=> Ok(x.clone()),
		});
	}
}
