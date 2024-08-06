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
	let ws: Box<Workspace> = unsafe { Box::from_raw(self_ptr as *mut Workspace) };
	let path = unsafe { env.get_string_unchecked(&input).expect("Couldn't get java string!") };
	RT.block_on(ws.create(path.to_str().expect("Not UTF-8"))).jexcept(&mut env);
}
