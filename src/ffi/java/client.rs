use jni::{objects::{JClass, JObject, JString}, sys::{jboolean, jlong, jobject, jobjectArray}, JNIEnv};
use crate::{api::Config, client::Client, ffi::java::{handle_error, null_check}, Workspace};

use super::{Deobjectify, JExceptable, JObjectify, tokio};

/// Connect using the given credentials to the default server, and return a [Client] to interact with it.
#[no_mangle]
pub extern "system" fn Java_mp_code_Client_connect<'local>(
	mut env: JNIEnv<'local>,
	_class: JClass<'local>,
	config: JObject<'local>	
) -> jobject {
	null_check!(env, config, std::ptr::null_mut());
	let config = Config::deobjectify(&mut env, config);
	if config.is_err() {
		handle_error!(&mut env, config, std::ptr::null_mut());
	}

	let client = tokio().block_on(Client::connect(config.unwrap()));
	if let Ok(client) = client {
		client.jobjectify(&mut env).jexcept(&mut env).as_raw()
	} else {
		handle_error!(&mut env, client, std::ptr::null_mut());
	}
}

/// Gets the current [crate::api::User].
#[no_mangle]
pub extern "system" fn Java_mp_code_Client_get_1user(
	mut env: JNIEnv,
	_class: JClass,
	self_ptr: jlong
) -> jobject {
	let client = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Client)) };
	client.user().clone()
		.jobjectify(&mut env)
		.jexcept(&mut env)
		.as_raw()
}

/// Join a [Workspace] and return a pointer to it.
#[no_mangle]
pub extern "system" fn Java_mp_code_Client_join_1workspace<'local>(
	mut env: JNIEnv<'local>,
	_class: JClass<'local>,
	self_ptr: jlong,
	workspace_id: JString<'local>
) -> jobject {
	null_check!(env, workspace_id, std::ptr::null_mut());
	let client = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Client)) };
	let workspace_id = unsafe { env.get_string_unchecked(&workspace_id) }
		.map(|wid| wid.to_string_lossy().to_string())
		.jexcept(&mut env);
	let workspace = tokio().block_on(client.join_workspace(workspace_id))
		.map(|workspace| spawn_updater(workspace.clone()));
	if let Ok(workspace) = workspace {
		workspace.jobjectify(&mut env).jexcept(&mut env).as_raw()
	} else {
		handle_error!(&mut env, workspace, std::ptr::null_mut())
	}
}

/// Create a workspace on server, if allowed to.
#[no_mangle]
pub extern "system" fn Java_mp_code_Client_create_1workspace<'local>(
	mut env: JNIEnv<'local>,
	_class: JClass<'local>,
	self_ptr: jlong,
	workspace_id: JString<'local>
) {
	null_check!(env, workspace_id, {});
	let client = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Client)) };
	let workspace_id = unsafe { env.get_string_unchecked(&workspace_id) }
		.map(|wid| wid.to_string_lossy().to_string())
		.jexcept(&mut env);
	tokio()
		.block_on(client.create_workspace(workspace_id))
		.jexcept(&mut env);
}

/// Delete a workspace on server, if allowed to.
#[no_mangle]
pub extern "system" fn Java_mp_code_Client_delete_1workspace<'local>(
	mut env: JNIEnv<'local>,
	_class: JClass<'local>,
	self_ptr: jlong,
	workspace_id: JString<'local>
) {
	null_check!(env, workspace_id, {});
	let client = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Client)) };
	let workspace_id = unsafe { env.get_string_unchecked(&workspace_id) }
		.map(|wid| wid.to_string_lossy().to_string())
		.jexcept(&mut env);
	tokio()
		.block_on(client.delete_workspace(workspace_id))
		.jexcept(&mut env);
}

/// Invite another user to an owned workspace.
#[no_mangle]
pub extern "system" fn Java_mp_code_Client_invite_1to_1workspace<'local>(
	mut env: JNIEnv<'local>,
	_class: JClass<'local>,
	self_ptr: jlong,
	workspace_id: JString<'local>,
	user: JString<'local>
) {
	null_check!(env, workspace_id, {});
	null_check!(env, user, {});
	let client = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Client)) };
	let workspace_id = unsafe { env.get_string_unchecked(&workspace_id) }
		.map(|wid| wid.to_string_lossy().to_string())
		.jexcept(&mut env);
	let user_name = unsafe { env.get_string_unchecked(&user) }
		.map(|wid| wid.to_string_lossy().to_string())
		.jexcept(&mut env);
	tokio()
		.block_on(client.invite_to_workspace(workspace_id, user_name))
		.jexcept(&mut env);
}

/// List available workspaces.
#[no_mangle]
pub extern "system" fn Java_mp_code_Client_list_1workspaces<'local>(
	mut env: JNIEnv<'local>,
	_class: JClass<'local>,
	self_ptr: jlong,
	owned: jboolean,
	invited: jboolean
) -> jobjectArray {
	let client = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Client)) };
	let list = tokio()
		.block_on(client.list_workspaces(owned != 0, invited != 0))
		.jexcept(&mut env);
	env.find_class("java/lang/String")
		.and_then(|class| env.new_object_array(list.len() as i32, class, JObject::null()))
		.inspect(|arr| {
			for (idx, path) in list.iter().enumerate() {
				env.new_string(path)
					.and_then(|path| env.set_object_array_element(arr, idx as i32, path))
					.jexcept(&mut env)
			}
		}).jexcept(&mut env).as_raw()
}

/// List available workspaces.
#[no_mangle]
pub extern "system" fn Java_mp_code_Client_active_1workspaces<'local>(
	mut env: JNIEnv<'local>,
	_class: JClass<'local>,
	self_ptr: jlong
) -> jobjectArray {
	let client = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Client)) };
	let list = client.active_workspaces();
	env.find_class("java/lang/String")
		.and_then(|class| env.new_object_array(list.len() as i32, class, JObject::null()))
		.inspect(|arr| {
			for (idx, path) in list.iter().enumerate() {
				env.new_string(path)
					.and_then(|path| env.set_object_array_element(arr, idx as i32, path))
					.jexcept(&mut env)
			}
		}).jexcept(&mut env).as_raw()
}

// TODO: this stays until we get rid of the arc then i'll have to find a better way
fn spawn_updater(workspace: Workspace) -> Workspace {
	let w = workspace.clone();
	tokio().spawn(async move {
		loop {
			tokio::time::sleep(std::time::Duration::from_secs(60)).await;
			w.fetch_buffers().await.unwrap();
			w.fetch_users().await.unwrap();
		}
	});
	workspace
}

/// Leave a [Workspace] and return whether or not the client was in such workspace.
#[no_mangle]
pub extern "system" fn Java_mp_code_Client_leave_1workspace<'local>(
	mut env: JNIEnv<'local>,
	_class: JClass<'local>,
	self_ptr: jlong,
	workspace_id: JString<'local>
) -> jboolean {
	null_check!(env, workspace_id, false as jboolean);
	let client = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Client)) };
	unsafe { env.get_string_unchecked(&workspace_id) }
		.map(|wid| wid.to_string_lossy().to_string())
		.map(|wid| client.leave_workspace(&wid) as jboolean)
		.jexcept(&mut env)
}

/// Get a [Workspace] by name and returns a pointer to it.
#[no_mangle]
pub extern "system" fn Java_mp_code_Client_get_1workspace<'local>(
	mut env: JNIEnv<'local>,
	_class: JClass<'local>,
	self_ptr: jlong,
	workspace_id: JString<'local>
) -> jobject {
	null_check!(env, workspace_id, std::ptr::null_mut());
	let client = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Client)) };
	let workspace_id = unsafe { env.get_string_unchecked(&workspace_id) }
		.map(|wid| wid.to_string_lossy().to_string())
		.jexcept(&mut env);
	if let Some(workspace) = client.get_workspace(&workspace_id) {
		workspace.jobjectify(&mut env).jexcept(&mut env).as_raw()
	} else {
		std::ptr::null_mut()
	}
}

/// Refresh the client's session token.
#[no_mangle]
pub extern "system" fn Java_mp_code_Client_refresh<'local>(
	mut env: JNIEnv<'local>,
	_class: JClass<'local>,
	self_ptr: jlong,
) {
	let client = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Client)) };
	tokio().block_on(client.refresh())
		.jexcept(&mut env);
}

/// Called by the Java GC to drop a [Client].
#[no_mangle]
pub extern "system" fn Java_mp_code_Client_free(_env: JNIEnv, _class: JClass, input: jlong) {
	let _ = unsafe { Box::from_raw(input as *mut Client) };
}
