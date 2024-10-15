use crate::prelude::*;
use mlua::prelude::*;
use mlua_codemp_patch as mlua;

use super::ext::a_sync::a_sync;

super::ext::impl_lua_serde! { CodempConfig CodempUser }

impl LuaUserData for CodempClient {
	fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| {
			Ok(format!("{:?}", this))
		});

		methods.add_method("current_user", |_, this, ()| {
			Ok(this.current_user().clone())
		});
		methods.add_method("active_workspaces", |_, this, ()| {
			Ok(this.active_workspaces())
		});

		methods.add_method(
			"refresh",
			|_, this, ()| a_sync! { this => this.refresh().await? },
		);

		methods.add_method(
			"attach_workspace",
			|_, this, (ws,): (String,)| a_sync! { this => this.attach_workspace(ws).await? },
		);

		methods.add_method(
			"create_workspace",
			|_, this, (ws,): (String,)| a_sync! { this => this.create_workspace(ws).await? },
		);

		methods.add_method(
			"delete_workspace",
			|_, this, (ws,): (String,)| a_sync! { this => this.delete_workspace(ws).await? },
		);

		methods.add_method("invite_to_workspace", |_, this, (ws,user):(String,String)|
			a_sync! { this => this.invite_to_workspace(ws, user).await? }
		);

		methods.add_method(
			"fetch_owned_workspaces",
			|_, this, ()| a_sync! { this => this.fetch_owned_workspaces().await? },
		);

		methods.add_method(
			"fetch_joined_workspaces",
			|_, this, ()| a_sync! { this => this.fetch_joined_workspaces().await? },
		);

		methods.add_method("leave_workspace", |_, this, (ws,): (String,)| {
			Ok(this.leave_workspace(&ws))
		});

		methods.add_method("get_workspace", |_, this, (ws,): (String,)| {
			Ok(this.get_workspace(&ws))
		});
	}
}
