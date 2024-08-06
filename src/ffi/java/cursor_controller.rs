use jni::{objects::{JClass, JObject, JValueGen}, sys::{jlong, jobject}, JNIEnv};
use crate::{api::Controller, ffi::java::util::JExceptable};

/*
 * Class:     mp_code_CursorController
 * Method:    recv
 * Signature: (J)Lmp/code/data/Cursor;
 */
#[no_mangle]
pub extern "system" fn Java_mp_code_CursorController_try_1recv(
	mut env: JNIEnv,
	_class: JClass,
	self_ptr: jlong,
) -> jobject {
	let controller = unsafe { Box::leak(Box::from_raw(self_ptr as *mut crate::cursor::Controller)) };
	match controller.try_recv().jexcept(&mut env) {
		None => JObject::null().as_raw(),
		Some(event) => {
			let class = env.find_class("mp/code/data/Cursor").expect("Couldn't find class!");
			env.new_object(
				class,
				"(IIIILjava/lang/String;Ljava/lang/String;)V",
				&[
					JValueGen::Int(event.start.0),
					JValueGen::Int(event.start.1),
					JValueGen::Int(event.end.0),
					JValueGen::Int(event.end.1),
					JValueGen::Object(&env.new_string(event.buffer).expect("Failed to create String!")),
					JValueGen::Object(&env.new_string(event.user.map(|x| x.to_string()).unwrap_or_default()).expect("Failed to create String!"))
				]
			).expect("failed creating object").into_raw()
		}
	}
}
// 	public Cursor(int startRow, int startCol, int endRow, int endCol, String buffer, String user) {

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
	todo!()
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
	let _ = unsafe { Box::from_raw(self_ptr as *mut crate::cursor::Controller) };
}
