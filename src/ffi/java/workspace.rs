use jni::{objects::{JClass, JObject, JString}, sys::{jboolean, jlong, jobject, jobjectArray, jstring}, JNIEnv};
use crate::Workspace;

use super::{handle_error, null_check, JExceptable, JObjectify};

/// Get the workspace id.
#[no_mangle]
pub extern "system" fn Java_mp_code_Workspace_get_1workspace_1id<'local>(
	mut env: JNIEnv<'local>,
	_class: JClass<'local>,
	self_ptr: jlong
) -> jstring {
	let workspace = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Workspace)) };
	env.new_string(workspace.id()).jexcept(&mut env).as_raw()
}

/// Get a cursor controller by name and returns a pointer to it.
#[no_mangle]
pub extern "system" fn Java_mp_code_Workspace_get_1cursor<'local>(
	mut env: JNIEnv<'local>,
	_class: JClass<'local>,
	self_ptr: jlong
) -> jobject {
	let workspace = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Workspace)) };
	workspace.cursor().jobjectify(&mut env).jexcept(&mut env).as_raw()
}

/// Get a buffer controller by name and returns a pointer to it.
#[no_mangle]
pub extern "system" fn Java_mp_code_Workspace_get_1buffer<'local>(
	mut env: JNIEnv<'local>,
	_class: JClass<'local>,
	self_ptr: jlong,
	path: JString<'local>
) -> jobject {
	null_check!(env, path, std::ptr::null_mut());
	let workspace = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Workspace)) };
	let path = unsafe { env.get_string_unchecked(&path) }
		.map(|path| path.to_string_lossy().to_string())
		.jexcept(&mut env);
	workspace.buffer_by_name(&path)
		.map(|buf| buf.jobjectify(&mut env).jexcept(&mut env))
		.unwrap_or_default()
		.as_raw()
}

/// Get the filetree.
#[no_mangle]
pub extern "system" fn Java_mp_code_Workspace_get_1file_1tree(
	mut env: JNIEnv,
	_class: JClass,
	self_ptr: jlong,
	filter: JString,
	strict: jboolean
) -> jobjectArray {
	let workspace = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Workspace)) };
	let filter: Option<String> = if filter.is_null() {
		None
	} else {
		Some(
			env.get_string(&filter)
				.map(|s| s.into())
				.jexcept(&mut env)
		)
	};

	let file_tree = workspace.filetree(filter.as_deref(), strict != 0);
	env.find_class("java/lang/String")
		.and_then(|class| env.new_object_array(file_tree.len() as i32, class, JObject::null()))
		.inspect(|arr| {
			for (idx, path) in file_tree.iter().enumerate() {
				env.new_string(path)
					.and_then(|path| env.set_object_array_element(arr, idx as i32, path))
					.jexcept(&mut env)
			}
		}).jexcept(&mut env).as_raw()
}

/// Gets a list of the active buffers.
#[no_mangle]
pub extern "system" fn Java_mp_code_Workspace_active_1buffers(
	mut env: JNIEnv,
	_class: JClass,
	self_ptr: jlong
) -> jobjectArray {
	let workspace = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Workspace)) };	
	let active_buffer_list = workspace.buffer_list();
	env.find_class("java/lang/String")
		.and_then(|class| env.new_object_array(active_buffer_list.len() as i32, class, JObject::null()))
		.inspect(|arr| {
			for (idx, path) in active_buffer_list.iter().enumerate() {
				env.new_string(path)
					.and_then(|path| env.set_object_array_element(arr, idx as i32, path))
					.jexcept(&mut env)
			}
		}).jexcept(&mut env).as_raw()
}

/// Create a new buffer.
#[no_mangle]
pub extern "system" fn Java_mp_code_Workspace_create_1buffer<'local>(
	mut env: JNIEnv,
	_class: JClass<'local>,
	self_ptr: jlong,
	path: JString<'local>
) {
	null_check!(env, path, {});
	let ws = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Workspace)) };
	let path = unsafe { env.get_string_unchecked(&path) }
		.map(|path| path.to_string_lossy().to_string())
		.jexcept(&mut env);
	super::tokio().block_on(ws.create(&path))
		.jexcept(&mut env);
}

/// Attach to a buffer and return a pointer to its [crate::buffer::Controller].
#[no_mangle]
pub extern "system" fn Java_mp_code_Workspace_attach_1to_1buffer<'local>(
	mut env: JNIEnv,
	_class: JClass<'local>,
	self_ptr: jlong,
	path: JString<'local>
) -> jobject {
	null_check!(env, path, std::ptr::null_mut());
	let workspace = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Workspace)) };
	let path = unsafe { env.get_string_unchecked(&path) }
		.map(|path| path.to_string_lossy().to_string())
		.jexcept(&mut env);
	super::tokio().block_on(workspace.attach(&path))
		.map(|buffer| buffer.jobjectify(&mut env).jexcept(&mut env))
		.jexcept(&mut env)
		.as_raw()
}

/// Detach from a buffer.
#[no_mangle]
pub extern "system" fn Java_mp_code_Workspace_detach_1from_1buffer<'local>(
	mut env: JNIEnv,
	_class: JClass<'local>,
	self_ptr: jlong,
	path: JString<'local>
) -> jobject {
	null_check!(env, path, std::ptr::null_mut());
	let workspace = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Workspace)) };
	let path = unsafe { env.get_string_unchecked(&path) }
		.map(|path| path.to_string_lossy().to_string())
		.jexcept(&mut env);
	workspace.detach(&path)
		.jobjectify(&mut env)
		.jexcept(&mut env)
		.as_raw()
}

/// Update the local buffer list.
#[no_mangle]
pub extern "system" fn Java_mp_code_Workspace_fetch_1buffers(
	mut env: JNIEnv,
	_class: JClass,
	self_ptr: jlong,
) {
	let workspace = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Workspace)) };
	super::tokio().block_on(workspace.fetch_buffers()).jexcept(&mut env);
}

/// Update the local user list.
#[no_mangle]
pub extern "system" fn Java_mp_code_Workspace_fetch_1users(
	mut env: JNIEnv,
	_class: JClass,
	self_ptr: jlong,
) {
	let workspace = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Workspace)) };
	super::tokio().block_on(workspace.fetch_users()).jexcept(&mut env);
}

/// List users attached to a buffer.
#[no_mangle]
pub extern "system" fn Java_mp_code_Workspace_list_1buffer_1users<'local>(
	mut env: JNIEnv,
	_class: JClass<'local>,
	self_ptr: jlong,
	path: JString<'local>,
) -> jobjectArray {
	null_check!(env, path, std::ptr::null_mut());
	let workspace = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Workspace)) };
	let buffer = unsafe { env.get_string_unchecked(&path) }
		.map(|buffer| buffer.to_string_lossy().to_string())
		.jexcept(&mut env);
	let users = super::tokio().block_on(workspace.list_buffer_users(&buffer))
		.jexcept(&mut env);

	if env.exception_check().unwrap_or(false) { // prevent illegal state
		return std::ptr::null_mut();
	}

	env.find_class("java/util/UUID")
		.and_then(|class| env.new_object_array(users.len() as i32, &class, JObject::null()))
		.inspect(|arr| {
			for (idx, user) in users.iter().enumerate() {
				user.id.jobjectify(&mut env)
					.and_then(|id| env.set_object_array_element(arr, idx as i32, id))
					.jexcept(&mut env);
			}
		}).jexcept(&mut env).as_raw()
}

/// Delete a buffer.
#[no_mangle]
pub extern "system" fn Java_mp_code_Workspace_delete_1buffer<'local>(
	mut env: JNIEnv,
	_class: JClass<'local>,
	self_ptr: jlong,
	path: JString<'local>,
) {
	null_check!(env, path, {});
	let workspace = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Workspace)) };
	let buffer = unsafe { env.get_string_unchecked(&path) }
		.map(|buffer| buffer.to_string_lossy().to_string())
		.jexcept(&mut env);
	super::tokio().block_on(workspace.delete(&buffer))
		.jexcept(&mut env);
}

/// Receive a workspace event if present.
#[no_mangle]
pub extern "system" fn Java_mp_code_Workspace_event(
	mut env: JNIEnv,
	_class: JClass,
	self_ptr: jlong
) -> jobject {
	let workspace = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Workspace)) };
	let event = super::tokio().block_on(workspace.event());
	if let Ok(event) = event {
		event.jobjectify(&mut env).jexcept(&mut env).as_raw()
	} else {
		handle_error!(&mut env, event, std::ptr::null_mut())
	}
}

/// Called by the Java GC to drop a [Workspace].
#[no_mangle]
pub extern "system" fn Java_mp_code_Workspace_free(_env: JNIEnv, _class: JClass, input: jlong) {
	let _ = unsafe { Box::from_raw(input as *mut Workspace) };
}
