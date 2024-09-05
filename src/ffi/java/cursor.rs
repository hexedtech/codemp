use jni::{objects::{JClass, JObject, JString, JValueGen}, sys::{jlong, jobject}, JNIEnv};
use crate::api::Controller;

use super::{JExceptable, RT};

/// Try to fetch a [crate::api::Cursor], or returns null if there's nothing.
#[no_mangle]
pub extern "system" fn Java_mp_code_CursorController_try_1recv(
	mut env: JNIEnv,
	_class: JClass,
	self_ptr: jlong,
) -> jobject {
	let controller = unsafe { Box::leak(Box::from_raw(self_ptr as *mut crate::cursor::Controller)) };
	let cursor = RT.block_on(controller.try_recv()).jexcept(&mut env);
	jni_recv(&mut env, cursor)
}

/// Block until it receives a [crate::api::Cursor].
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
		None => JObject::default(),
		Some(event) => {
			env.find_class("mp/code/data/Cursor")
				.and_then(|class| {
					let buffer = env.new_string(&event.buffer).jexcept(env);
					let user = event.user
						.map(|uuid| uuid.to_string())
						.map(|user| env.new_string(user).jexcept(env))
						.unwrap_or_default();
					env.new_object(
						class,
						"(IIIILjava/lang/String;Ljava/lang/String;)V",
						&[
							JValueGen::Int(event.start.0),
							JValueGen::Int(event.start.1),
							JValueGen::Int(event.end.0),
							JValueGen::Int(event.end.1),
							JValueGen::Object(&buffer),
							JValueGen::Object(&user)
						]
					)
				}).jexcept(env)
		}
	}.as_raw()
}

/// Receive from Java, converts and sends a [crate::api::Cursor].
#[no_mangle]
pub extern "system" fn Java_mp_code_CursorController_send<'local>(
	mut env: JNIEnv,
	_class: JClass<'local>,
	self_ptr: jlong,
	input: JObject<'local>,
) {
	let start_row = env.get_field(&input, "startRow", "I")
		.and_then(|sr| sr.i())
		.jexcept(&mut env);
	let start_col = env.get_field(&input, "startCol", "I")
		.and_then(|sc| sc.i())
		.jexcept(&mut env);
	let end_row = env.get_field(&input, "endRow", "I")
		.and_then(|er| er.i())
		.jexcept(&mut env);
	let end_col = env.get_field(&input, "endCol", "I")
		.and_then(|ec| ec.i())
		.jexcept(&mut env);

	let buffer = env.get_field(&input, "buffer", "Ljava/lang/String;")
		.and_then(|b| b.l())
		.map(|b| b.into())
		.jexcept(&mut env);
	let buffer = env.get_string(&buffer)
		.map(|b| b.into())
		.jexcept(&mut env);

	let user: JString = env.get_field(&input, "user", "Ljava/lang/String;")
		.and_then(|u| u.l())
		.map(|u| u.into())
		.jexcept(&mut env);
	let user = if user.is_null() {
		None
	} else {
		Some(env.get_string(&user)
			.map(|u| u.into())
			.jexcept(&mut env)
		)
	};

	let controller = unsafe { Box::leak(Box::from_raw(self_ptr as *mut crate::cursor::Controller)) };
	RT.block_on(controller.send(crate::api::Cursor {
		start: (start_row, start_col),
		end: (end_row, end_col),
		buffer,
		user
	})).jexcept(&mut env);
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
