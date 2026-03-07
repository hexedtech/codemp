use crate::prelude::*;
use mlua::prelude::*;

use super::ext::a_sync::a_sync;

super::ext::impl_lua_serde! { CodempConfig CodempUserInfo }

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
			|_, this, (user, workspace): (String, String)| {
				a_sync! { this => this.attach_workspace(user, workspace).await? }
			},
		);

		methods.add_method(
			"create_workspace",
			|_, this, (ws,): (String,)| a_sync! { this => this.create_workspace(ws).await? },
		);

		methods.add_method(
			"delete_workspace",
			|_, this, (ws,): (String,)| a_sync! { this => this.delete_workspace(ws).await? },
		);

		methods.add_method(
			"quit_workspace",
			|_, this, (user, workspace): (String, String)| {
				a_sync! {
					this => this.quit_workspace(user, workspace).await?
				}
			},
		);

		methods.add_method(
			"accept_invite",
			|_, this, (user, workspace): (String, String)| {
				a_sync! {
					this => this.accept_invite(user, workspace).await?
				}
			},
		);

		methods.add_method(
			"reject_invite",
			|_, this, (user, workspace): (String, String)| {
				a_sync! {
					this => this.reject_invite(user, workspace).await?
				}
			},
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

		methods.add_method(
			"leave_workspace",
			|_, this, (user, workspace): (String, String)| Ok(this.leave_workspace(user, workspace)),
		);

		methods.add_method(
			"get_workspace",
			|_, this, (user, workspace): (String, String)| Ok(this.get_workspace(user, workspace)),
		);

		methods.add_method("get_user_info", |_, this, (user,): (String,)| {
			a_sync! {
				this => crate::api::UserInfo::from(this.get_user_info(user).await?)
			}
		});

		// TODO need to derive ser/de on Event, but this is in protobuf...
		// methods.add_method("recv", |_, this, ()| a_sync! { this => this.recv().await? });

		// methods.add_method(
		// 	"try_recv",
		// 	|_, this, ()| a_sync! { this => this.try_recv().await? },
		// );

		methods.add_method("poll", |_, this, ()| a_sync! { this => this.poll().await? });

		methods.add_method("callback", |lua, this, (cb,): (LuaFunction,)| {
			let key = this.lua_callback_id();
			lua.set_named_registry_value(&key, cb)?;
			Ok(this.callback(move |controller: CodempClient| {
				super::ext::callback().invoke(key.clone(), controller, false)
			}))
		});

		methods.add_method("clear_callback", |lua, this, ()| {
			this.clear_callback();
			lua.unset_named_registry_value(&this.lua_callback_id())
		});
	}
}

impl CodempClient {
	fn lua_callback_id(&self) -> String {
		format!(
			"codemp-client({})-callback-registry",
			self.current_user().name
		)
	}
}
