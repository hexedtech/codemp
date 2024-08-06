use jni::{objects::{JClass, JString}, sys::{jlong, jstring}, JNIEnv};
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
	_env: JNIEnv<'local>,
	_class: JClass<'local>,
	self_ptr: jlong
) -> jlong {
	let workspace  = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Workspace)) };
	Box::into_raw(Box::new(workspace.cursor())) as jlong
}

/// Gets a buffer controller by name and returns a pointer to it.
#[no_mangle]
pub extern "system" fn Java_mp_code_Workspace_get_1buffer<'local>(
	env: JNIEnv<'local>,
	_class: JClass<'local>,
	self_ptr: jlong,
	input: JString<'local>
) -> jlong {
	let workspace  = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Workspace)) };
	let path = unsafe { env.get_string_unchecked(&input).expect("Couldn't get java string!") };
	Box::into_raw(Box::new(workspace.buffer_by_name(path.to_str().expect("Not UTF-8")))) as jlong
}

/// Creates a [Buffer]
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

/*
 * Class:     mp_code_Workspace
 * Method:    get_file_tree
 * Signature: (J)V
 */
#[no_mangle]
pub extern "system" fn Java_mp_code_Workspace_get_1file_1tree<'local>(
	mut env: JNIEnv,
	_class: JClass<'local>,
	self_ptr: jlong,
) {
	let workspace  = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Workspace)) };
    let file_tree = workspace.filetree();
    todo!() // how to return Vec<String> ? []String ?
}

/*
 * Class:     mp_code_Workspace
 * Method:    attach_to_buffer
 * Signature: (J)J
 */
#[no_mangle]
pub extern "system" fn Java_mp_code_Workspace_attach_1to_1buffer<'local>(
	mut env: JNIEnv,
	_class: JClass<'local>,
	_self_ptr: jlong,
) -> jlong {
    todo!()
}

/*
 * Class:     mp_code_Workspace
 * Method:    fetch_buffers
 * Signature: (J)V
 */
#[no_mangle]
pub extern "system" fn Java_mp_code_Workspace_fetch_1buffers<'local>(
	mut env: JNIEnv,
	_class: JClass<'local>,
	self_ptr: jlong,
) {
	let workspace  = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Workspace)) };
    RT.block_on(workspace.fetch_buffers()).jexcept(&mut env);
}

/*
 * Class:     mp_code_Workspace
 * Method:    fetch_users
 * Signature: (J)V
 */
#[no_mangle]
pub extern "system" fn Java_mp_code_Workspace_fetch_1users<'local>(
	mut env: JNIEnv,
	_class: JClass<'local>,
	self_ptr: jlong,
) {
	let workspace  = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Workspace)) };
    RT.block_on(workspace.fetch_users()).jexcept(&mut env);
}

/*
 * Class:     mp_code_Workspace
 * Method:    list_buffer_users
 * Signature: (JLjava/lang/String;)V
 */
#[no_mangle]
pub extern "system" fn Java_mp_code_Workspace_list_1buffer_1users<'local>(
	mut env: JNIEnv,
	_class: JClass<'local>,
	self_ptr: jlong,
	input: JString<'local>,
) {
	let workspace  = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Workspace)) };
	let buffer : String = unsafe {
		env.get_string_unchecked(&input)
			.expect("Couldn't get java string!")
			.into()
	};
	let users = RT.block_on(workspace.list_buffer_users(&buffer))
		.jexcept(&mut env)
		.into_iter()
		.map(|x| x.id.to_string())
		.collect::<Vec<String>>();
	todo!() // how to return Vec<String>?
}

/*
 * Class:     mp_code_Workspace
 * Method:    delete_buffer
 * Signature: (JLjava/lang/String;)V
 */
#[no_mangle]
pub extern "system" fn Java_mp_code_Workspace_delete_1buffer<'local>(
	mut env: JNIEnv,
	_class: JClass<'local>,
	self_ptr: jlong,
	input: JString<'local>,
) {
	let buffer : String = unsafe {
		env.get_string_unchecked(&input)
			.expect("Couldn't get java string!")
			.into()
	};
	let workspace  = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Workspace)) };
	RT.block_on(workspace.delete(&buffer)).jexcept(&mut env);
}
