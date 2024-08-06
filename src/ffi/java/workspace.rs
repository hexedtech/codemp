use jni::{objects::{JClass, JString}, sys::jlong, JNIEnv};
use crate::{Client, Workspace};

use super::{util::JExceptable, RT};

/// Called by the Java GC to drop a [Workspace].
#[no_mangle]
pub extern "system" fn Java_mp_code_Workspace_free(_env: JNIEnv, _class: JClass, input: jlong) {
	super::util::dereference_and_drop::<Client>(input)
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
	let path: String = env.get_string(&input).expect("Couldn't get java string!").into();
	RT.block_on(ws.create(&path)).jexcept(&mut env);
}
