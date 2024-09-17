use jni::{objects::{JClass, JObject, JString, JValueGen}, sys::{jboolean, jint, jlong, jobject, jobjectArray}, JNIEnv};
use crate::{api::Config, client::Client, Workspace};

use super::{JExceptable, JObjectify};

/// Connect using the given credentials to the default server, and return a [Client] to interact with it.
#[no_mangle]
pub extern "system" fn Java_mp_code_Client_connect<'local>(
	mut env: JNIEnv,
	_class: JClass<'local>,
	user: JString<'local>,
	pwd: JString<'local>
) -> jobject {
	let username: String = env.get_string(&user)
		.map(|s| s.into())
		.jexcept(&mut env);
	let password: String = env.get_string(&pwd)
		.map(|s| s.into())
		.jexcept(&mut env);
	connect_internal(env, Config {
		username,
		password,
		host: None,
		port: None,
		tls: None
	})
}

/// Connect to a given URL and return a [Client] to interact with that server.
#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_mp_code_Client_connectToServer<'local>(
	mut env: JNIEnv,
	_class: JClass<'local>,
	user: JString<'local>,
	pwd: JString<'local>,
	host: JString<'local>,
	port: jint,
	tls: jboolean
) -> jobject {
	let username: String = env.get_string(&user)
		.map(|s| s.into())
		.jexcept(&mut env);
	let password: String = env.get_string(&pwd)
		.map(|s| s.into())
		.jexcept(&mut env);
	let host: String = env.get_string(&host)
		.map(|s| s.into())
		.jexcept(&mut env);

	if port < 0 {
		env.throw_new("mp/code/exceptions/JNIException", "Negative port number!")
			.jexcept(&mut env);
	}

	connect_internal(env, Config {
		username,
		password,
		host: Some(host),
		port: Some(port as u16),
		tls: Some(tls != 0),
	})
}

fn connect_internal(mut env: JNIEnv, config: Config) -> jobject {
	super::tokio().block_on(Client::connect(config))
		.map(|client| Box::into_raw(Box::new(client)) as jlong)
		.map(|ptr| {
			env.find_class("mp/code/Client")
				.and_then(|class| env.new_object(class, "(J)V", &[JValueGen::Long(ptr)]))
				.jexcept(&mut env)
		}).jexcept(&mut env).as_raw()
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
	input: JString<'local>
) -> jobject {
	let client = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Client)) };
	let workspace_id = unsafe { env.get_string_unchecked(&input) }
		.map(|wid| wid.to_string_lossy().to_string())
		.jexcept(&mut env);
	super::tokio().block_on(client.join_workspace(workspace_id))
		.map(|workspace| spawn_updater(workspace.clone()))
		.map(|workspace| Box::into_raw(Box::new(workspace)) as jlong)
		.map(|ptr| {
			env.find_class("mp/code/Workspace")
				.and_then(|class| env.new_object(class, "(J)V", &[JValueGen::Long(ptr)]))
				.jexcept(&mut env)
		}).jexcept(&mut env).as_raw()
}

/// Create a workspace on server, if allowed to.
#[no_mangle]
pub extern "system" fn Java_mp_code_Client_create_1workspace<'local>(
	mut env: JNIEnv<'local>,
	_class: JClass<'local>,
	self_ptr: jlong,
	input: JString<'local>
) {
	let client = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Client)) };
	let workspace_id = unsafe { env.get_string_unchecked(&input) }
		.map(|wid| wid.to_string_lossy().to_string())
		.jexcept(&mut env);
	super::tokio()
		.block_on(client.create_workspace(workspace_id))
		.jexcept(&mut env);
}

/// Delete a workspace on server, if allowed to.
#[no_mangle]
pub extern "system" fn Java_mp_code_Client_delete_1workspace<'local>(
	mut env: JNIEnv<'local>,
	_class: JClass<'local>,
	self_ptr: jlong,
	input: JString<'local>
) {
	let client = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Client)) };
	let workspace_id = unsafe { env.get_string_unchecked(&input) }
		.map(|wid| wid.to_string_lossy().to_string())
		.jexcept(&mut env);
	super::tokio()
		.block_on(client.delete_workspace(workspace_id))
		.jexcept(&mut env);
}

/// Invite another user to an owned workspace.
#[no_mangle]
pub extern "system" fn Java_mp_code_Client_invite_1to_1workspace<'local>(
	mut env: JNIEnv<'local>,
	_class: JClass<'local>,
	self_ptr: jlong,
	ws: JString<'local>,
	usr: JString<'local>
) {
	let client = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Client)) };
	let workspace_id = unsafe { env.get_string_unchecked(&ws) }
		.map(|wid| wid.to_string_lossy().to_string())
		.jexcept(&mut env);
	let user_name = unsafe { env.get_string_unchecked(&usr) }
		.map(|wid| wid.to_string_lossy().to_string())
		.jexcept(&mut env);
	super::tokio()
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
	let list = super::tokio()
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
	super::tokio().spawn(async move {
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
	input: JString<'local>
) -> jboolean {
	let client = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Client)) };
	unsafe { env.get_string_unchecked(&input) }
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
	input: JString<'local>
) -> jobject {
	let client = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Client)) };
	let workspace_id = unsafe { env.get_string_unchecked(&input) }
		.map(|wid| wid.to_string_lossy().to_string())
		.jexcept(&mut env);
	client.get_workspace(&workspace_id)
		.map(|workspace| Box::into_raw(Box::new(workspace)) as jlong)
		.map(|ptr| {
			env.find_class("mp/code/Workspace")
				.and_then(|class| env.new_object(class, "(J)V", &[JValueGen::Long(ptr)]))
				.jexcept(&mut env)
		}).unwrap_or_default().as_raw()
}

/// Refresh the client's session token.
#[no_mangle]
pub extern "system" fn Java_mp_code_Client_refresh<'local>(
	mut env: JNIEnv<'local>,
	_class: JClass<'local>,
	self_ptr: jlong,
) {
	let client = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Client)) };
	super::tokio().block_on(client.refresh())
		.jexcept(&mut env);
}

/// Called by the Java GC to drop a [Client].
#[no_mangle]
pub extern "system" fn Java_mp_code_Client_free(_env: JNIEnv, _class: JClass, input: jlong) {
	let _ = unsafe { Box::from_raw(input as *mut Client) };
}
