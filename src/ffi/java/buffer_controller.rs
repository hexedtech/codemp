use jni::{objects::{JClass, JObject}, sys::{jlong, jobject, jstring}, JNIEnv};

use crate::{api::Controller, buffer::Controller};

use super::util::JExceptable;

/*
 * Class:     mp_code_BufferController
 * Method:    get_name
 * Signature: (J)Ljava/lang/String;
 */
#[no_mangle]
pub extern "system" fn Java_mp_code_BufferController_get_1name<'local>(
	env: JNIEnv,
	_class: JClass<'local>,
	self_ptr: jlong,
) -> jstring {
	let controller = unsafe { Box::leak(Box::from_raw(self_ptr as *mut crate::buffer::Controller)) };
	let content = controller.name();
	env.new_string(content)
		.expect("could not create jstring")
		.as_raw()
}

/*
 * Class:     mp_code_BufferController
 * Method:    get_content
 * Signature: (J)Ljava/lang/String;
 */
#[no_mangle]
pub extern "system" fn Java_mp_code_BufferController_get_1content<'local>(
	env: JNIEnv,
	_class: JClass<'local>,
	self_ptr: jlong,
) -> jstring {
	let controller = unsafe { Box::leak(Box::from_raw(self_ptr as *mut crate::buffer::Controller)) };
	let content = controller.content();
	env.new_string(content)
		.expect("could not create jstring")
		.as_raw()
}

/*
 * Class:     mp_code_BufferController
 * Method:    try_recv
 * Signature: (J)Lmp/code/data/TextChange;
 */
#[no_mangle]
pub extern "system" fn Java_mp_code_BufferController_try_1recv<'local>(
	mut env: JNIEnv,
	_class: JClass<'local>,
	self_ptr: jlong,
) -> jobject {
	let controller = unsafe { Box::leak(Box::from_raw(self_ptr as *mut crate::buffer::Controller)) };
	let change = controller.try_recv().jexcept(&mut env);
	todo!()
}

/*
 * Class:     mp_code_BufferController
 * Method:    send
 * Signature: (JLmp/code/data/TextChange;)V
 */
#[no_mangle]
pub extern "system" fn Java_mp_code_BufferController_send<'local>(
	mut env: JNIEnv,
	_class: JClass<'local>,
	self_ptr: jlong,
	input: JObject<'local>
) {
	todo!()
}

/*
 * Class:     mp_code_BufferController
 * Method:    free
 * Signature: (J)V
 */
#[no_mangle]
pub extern "system" fn Java_mp_code_BufferController_free<'local>(
	_env: JNIEnv,
	_class: JClass<'local>,
	self_ptr: jlong,
) {
	unsafe { Box::from_raw(self_ptr as *mut crate::cursor::Controller) };
}

