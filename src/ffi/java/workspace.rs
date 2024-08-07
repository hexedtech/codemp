use jni::{objects::{JClass, JObject, JString, JValueGen}, sys::{jlong, jobject, jobjectArray, jstring}, JNIEnv};
use crate::Workspace;

use super::{util::JExceptable, RT};

/// Called by the Java GC to drop a [Workspace].
#[no_mangle]
pub extern "system" fn Java_mp_code_Workspace_free(_env: JNIEnv, _class: JClass, input: jlong) {
	let _ = unsafe { Box::from_raw(input as *mut Workspace) };
}

/// Gets the workspace id.
#[no_mangle]
pub extern "system" fn Java_mp_code_Workspace_get_1workspace_1id<'local>(
	env: JNIEnv<'local>,
	_class: JClass<'local>,
	self_ptr: jlong
) -> jstring {
	let workspace  = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Workspace)) };
	env.new_string(workspace.id())
		.expect("Failed to convert to Java String!")
		.as_raw()
}

/// Gets a cursor controller by name and returns a pointer to it.
#[no_mangle]
pub extern "system" fn Java_mp_code_Workspace_get_1cursor<'local>(
	mut env: JNIEnv<'local>,
	_class: JClass<'local>,
	self_ptr: jlong
) -> jobject {
	let workspace  = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Workspace)) };
	let class = env.find_class("mp/code/CursorController").expect("Failed to find class");
	env.new_object(class, "(J)V", &[JValueGen::Long(Box::into_raw(Box::new(workspace.cursor())) as jlong)])
		.expect("Failed to initialise object")
		.as_raw()
}

/// Gets a buffer controller by name and returns a pointer to it.
#[no_mangle]
pub extern "system" fn Java_mp_code_Workspace_get_1buffer<'local>(
	mut env: JNIEnv<'local>,
	_class: JClass<'local>,
	self_ptr: jlong,
	input: JString<'local>
) -> jobject {
	let workspace  = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Workspace)) };
	let path = unsafe { env.get_string_unchecked(&input).expect("Couldn't get java string!") };
	if let Some(buf) = workspace.buffer_by_name(path.to_str().expect("Not UTF-8!")) {
		let class = env.find_class("mp/code/BufferController").expect("Failed to find class");
		env.new_object(class, "(J)V", &[JValueGen::Long(Box::into_raw(Box::new(buf)) as jlong)])
			.expect("Failed to initialise object")
			.as_raw()
	} else {
		JObject::null().as_raw()
	}
}

/// Creates a new buffer.
#[no_mangle]
pub extern "system" fn Java_mp_code_Workspace_create_1buffer<'local>(
	mut env: JNIEnv,
	_class: JClass<'local>,
	self_ptr: jlong,
	input: JString<'local>
) {
	let ws = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Workspace)) };
	let path = unsafe { env.get_string_unchecked(&input).expect("Couldn't get java string!") };
	RT.block_on(ws.create(path.to_str().expect("Not UTF-8"))).jexcept(&mut env);
}

/// Gets the filetree.
#[no_mangle]
pub extern "system" fn Java_mp_code_Workspace_get_1file_1tree(
	mut env: JNIEnv,
	_class: JClass,
	self_ptr: jlong,
) -> jobjectArray {
	let workspace  = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Workspace)) };
	let file_tree = workspace.filetree();
	let class = env.find_class("java/lang/String").expect("Failed to find class!");
	let arr = env.new_object_array(file_tree.len() as i32, class, JObject::null())
		.expect("failed creating array");
	for (idx, path) in file_tree.iter().enumerate() {
		let js = env.new_string(path).expect("Failed to create String!");
		env.set_object_array_element(&arr, idx as i32, js)
			.expect("Failed to set array element!")
	}

	arr.as_raw()
}

/// Attaches to a buffer and returns a pointer to its [crate::buffer::Controller].
#[no_mangle]
pub extern "system" fn Java_mp_code_Workspace_attach_1to_1buffer<'local>(
	mut env: JNIEnv,
	_class: JClass<'local>,
	self_ptr: jlong,
	input: JString<'local>
) -> jobject {
	let workspace  = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Workspace)) };
	let path = unsafe { env.get_string_unchecked(&input).expect("Couldn't get java string!") };
	RT.block_on(workspace.attach(path.to_str().expect("Not UTF-8!")))
		.map(|buffer| Box::into_raw(Box::new(buffer)) as jlong)
		.map(|ptr| {
			let class = env.find_class("mp/code/BufferController").expect("Failed to find class");
			env.new_object(class, "(J)V", &[JValueGen::Long(ptr)])
			.expect("Failed to initialise object")
		}).jexcept(&mut env).as_raw()
}

/// Updates the local buffer list.
#[no_mangle]
pub extern "system" fn Java_mp_code_Workspace_fetch_1buffers(
	mut env: JNIEnv,
	_class: JClass,
	self_ptr: jlong,
) {
	let workspace  = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Workspace)) };
	RT.block_on(workspace.fetch_buffers()).jexcept(&mut env);
}

/// Updates the local user list.
#[no_mangle]
pub extern "system" fn Java_mp_code_Workspace_fetch_1users(
	mut env: JNIEnv,
	_class: JClass,
	self_ptr: jlong,
) {
	let workspace  = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Workspace)) };
	RT.block_on(workspace.fetch_users()).jexcept(&mut env);
}

/// Lists users attached to a buffer.
#[no_mangle]
pub extern "system" fn Java_mp_code_Workspace_list_1buffer_1users<'local>(
	mut env: JNIEnv,
	_class: JClass<'local>,
	self_ptr: jlong,
	input: JString<'local>,
) -> jobjectArray {
	let workspace  = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Workspace)) };
	let buffer = unsafe { env.get_string_unchecked(&input).expect("Couldn't get java string!") };
	let users = RT.block_on(workspace.list_buffer_users(buffer.to_str().expect("Not UTF-8!")))
		.jexcept(&mut env);

	let class = env.find_class("java/lang/String").expect("Failed to find class!");
	let arr = env.new_object_array(users.len() as i32, class, JObject::null())
		.expect("failed creating array");

	for (idx, user) in users.iter().enumerate() {
		let js = env.new_string(&user.id).expect("Failed to create String!");
		env.set_object_array_element(&arr, idx as i32, js)
			.expect("Failed to set array element!")
	}

	arr.as_raw()
}

/// Deletes a buffer.
#[no_mangle]
pub extern "system" fn Java_mp_code_Workspace_delete_1buffer<'local>(
	mut env: JNIEnv,
	_class: JClass<'local>,
	self_ptr: jlong,
	input: JString<'local>,
) {
	let buffer = unsafe { env.get_string_unchecked(&input).expect("Couldn't get java string!") };
	let workspace  = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Workspace)) };
	RT.block_on(workspace.delete(buffer.to_str().expect("Not UTF-8!"))).jexcept(&mut env);
}

/// Polls a list of buffers, returning the first ready one.
#[no_mangle]
pub extern "system" fn Java_mp_code_Workspace_select_1buffer(
	mut env: JNIEnv,
	_class: JClass,
	self_ptr: jlong,
	timeout: jlong
) -> jobject {
	let workspace  = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Workspace)) };
	let buffers = workspace.buffer_list();
	let mut controllers = Vec::default();
	for buffer in buffers {
		if let Some(controller) = workspace.buffer_by_name(&buffer) {
			controllers.push(controller);
		}
	}

	let active = RT.block_on(crate::buffer::tools::select_buffer(
		&controllers,
		Some(std::time::Duration::from_millis(timeout as u64)),
		&RT,
	)).jexcept(&mut env);

	if let Some(buf) = active {
		let class = env.find_class("mp/code/BufferController").expect("Failed to find class");
		env.new_object(class, "(J)V", &[JValueGen::Long(Box::into_raw(Box::new(buf)) as jlong)])
			.expect("Failed to initialise object")
			.as_raw()
	} else {
		JObject::null().as_raw()
	}
}
