use mlua_codemp_patch as mlua;
use mlua::prelude::*;
use crate::prelude::*;

use super::ext::a_sync::a_sync;
use super::ext::from_lua_serde;

impl LuaUserData for CodempClient {
	fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("id", |_, this| Ok(this.user().id.to_string()));
		fields.add_field_method_get("username", |_, this| Ok(this.user().name.clone()));
		fields.add_field_method_get("active_workspaces", |_, this| Ok(this.active_workspaces()));
	}

	fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(format!("{:?}", this)));

		methods.add_method("refresh", |_, this, ()|
			a_sync! { this => this.refresh().await? }
		);

		methods.add_method("join_workspace", |_, this, (ws,):(String,)|
			a_sync! { this => this.join_workspace(ws).await? }
		);

		methods.add_method("create_workspace", |_, this, (ws,):(String,)|
			a_sync! { this => this.create_workspace(ws).await? }
		);

		methods.add_method("delete_workspace", |_, this, (ws,):(String,)|
			a_sync! { this => this.delete_workspace(ws).await? }
		);

		methods.add_method("invite_to_workspace", |_, this, (ws,user):(String,String)|
			a_sync! { this => this.invite_to_workspace(ws, user).await? }
		);

		methods.add_method("list_workspaces", |_, this, (owned,invited):(Option<bool>,Option<bool>)|
			a_sync! { this => this.list_workspaces(owned.unwrap_or(true), invited.unwrap_or(true)).await? }
		);

		methods.add_method("leave_workspace", |_, this, (ws,):(String,)|
			Ok(this.leave_workspace(&ws))
		);
		
		methods.add_method("get_workspace", |_, this, (ws,):(String,)| Ok(this.get_workspace(&ws)));
	}
}

from_lua_serde! { CodempConfig }
impl LuaUserData for CodempConfig {
	fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("username", |_, this| Ok(this.username.clone()));
		fields.add_field_method_get("password", |_, this| Ok(this.password.clone()));
		fields.add_field_method_get("host", |_, this| Ok(this.host.clone()));
		fields.add_field_method_get("port", |_, this| Ok(this.port));
		fields.add_field_method_get("tls", |_, this| Ok(this.tls));
	}
}
