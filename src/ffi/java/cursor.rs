use jni::{objects::{JClass, JObject, JString, JValueGen}, sys::{jlong, jobject}, JNIEnv};
use crate::{api::Controller, ffi::java::util::JExceptable};

use super::RT;

/// Tries to fetch a [crate::api::Cursor], or returns null if there's nothing.
#[no_mangle]
pub extern "system" fn Java_mp_code_CursorController_try_1recv(
	mut env: JNIEnv,
	_class: JClass,
	self_ptr: jlong,
) -> jobject {
	let controller = unsafe { Box::leak(Box::from_raw(self_ptr as *mut crate::cursor::Controller)) };
	let cursor = controller.try_recv().jexcept(&mut env);
	jni_recv(&mut env, cursor)
}

/// Blocks until it receives a [crate::api::Cursor].
#[no_mangle]
pub extern "system" fn Java_mp_code_CursorController_recv(
	mut env: JNIEnv,
	_class: JClass,
	self_ptr: jlong,
) -> jobject {
	let controller = unsafe { Box::leak(Box::from_raw(self_ptr as *mut crate::cursor::Controller)) };
	let cursor = RT.block_on(controller.recv()).map(Some).jexcept(&mut env);
	jni_recv(&mut env, cursor)
}

/// Utility method to convert a [crate::api::Cursor] to its Java equivalent.
fn jni_recv(env: &mut JNIEnv, cursor: Option<crate::api::Cursor>) -> jobject {
	match cursor {
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

/// Receives from Java, converts and sends a [crate::api::Cursor].
#[no_mangle]
pub extern "system" fn Java_mp_code_CursorController_send<'local>(
	mut env: JNIEnv,
	_class: JClass<'local>,
	self_ptr: jlong,
	input: JObject<'local>,
) {
	let start_row = env.get_field(&input, "startRow", "I").expect("could not get field").i().expect("field was not of expected type");
	let start_col = env.get_field(&input, "startCol", "I").expect("could not get field").i().expect("field was not of expected type");
	let end_row = env.get_field(&input, "endRow", "I").expect("could not get field").i().expect("field was not of expected type");
	let end_col = env.get_field(&input, "endCol", "I").expect("could not get field").i().expect("field was not of expected type");

	let buffer = env.get_field(&input, "buffer", "Ljava/lang/String;")
		.expect("could not get field")
		.l()
		.expect("field was not of expected type")
		.into();
	let buffer = env.get_string(&buffer).expect("Failed to get String!").into();
	
	let user: JString = env.get_field(&input, "user", "Ljava/lang/String;")
		.expect("could not get field")
		.l()
		.expect("field was not of expected type")
		.into();
	let user = if user.is_null() {
		None
	} else {
		let jstring = env.get_string(&user).expect("Failed to get String!");
		Some(uuid::Uuid::parse_str(jstring.to_str().expect("Not valid UTF-8")).expect("Invalid UUI!"))
	};

	let controller = unsafe { Box::leak(Box::from_raw(self_ptr as *mut crate::cursor::Controller)) };
	controller.send(crate::api::Cursor {
		start: (start_row, start_col),
		end: (end_row, end_col),
		buffer,
		user
	}).jexcept(&mut env);
}

/// Called by the Java GC to drop a [crate::cursor::Controller].
#[no_mangle]
pub extern "system" fn Java_mp_code_CursorController_free(
	_env: JNIEnv,
	_class: JClass,
	self_ptr: jlong,
) {
	let _ = unsafe { Box::from_raw(self_ptr as *mut crate::cursor::Controller) };
}
