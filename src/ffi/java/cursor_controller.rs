use jni::{objects::{JClass, JObject}, sys::{jlong, jobject}, JNIEnv};

/*
 * Class:     mp_code_CursorController
 * Method:    recv
 * Signature: (J)Lmp/code/data/Cursor;
 */
#[no_mangle]
pub extern "system" fn Java_mp_code_CursorController_recv<'local>(
	mut env: JNIEnv,
	_class: JClass<'local>,
	self_ptr: jlong,
) -> jobject {
	todo!()
}

/*
 * Class:     mp_code_CursorController
 * Method:    send
 * Signature: (JLmp/code/data/Cursor;)V
 */
#[no_mangle]
pub extern "system" fn Java_mp_code_CursorController_send<'local>(
	mut env: JNIEnv,
	_class: JClass<'local>,
	self_ptr: jlong,
	input: JObject<'local>,
) {

}

/*
 * Class:     mp_code_CursorController
 * Method:    free
 * Signature: (J)V
 */
#[no_mangle]
pub extern "system" fn Java_mp_code_CursorController_free<'local>(
	_env: JNIEnv,
	_class: JClass<'local>,
	self_ptr: jlong,
) {
	unsafe { Box::from_raw(self_ptr as *mut crate::cursor::Controller) };
}
