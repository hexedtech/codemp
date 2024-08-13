use jni::{objects::{JClass, JObject, JValueGen}, sys::{jlong, jobject, jstring}, JNIEnv};

use crate::api::Controller;

use super::{JExceptable, RT};

/// Gets the name of the buffer. 
#[no_mangle]
pub extern "system" fn Java_mp_code_BufferController_get_1name(
	mut env: JNIEnv,
	_class: JClass,
	self_ptr: jlong,
) -> jstring {
	let controller = unsafe { Box::leak(Box::from_raw(self_ptr as *mut crate::buffer::Controller)) };
	let content = controller.name();
	env.new_string(content)
		.jexcept(&mut env)
		.as_raw()
}

/// Gets the contents of the buffers.
#[no_mangle]
pub extern "system" fn Java_mp_code_BufferController_get_1content(
	mut env: JNIEnv,
	_class: JClass,
	self_ptr: jlong,
) -> jstring {
	let controller = unsafe { Box::leak(Box::from_raw(self_ptr as *mut crate::buffer::Controller)) };
	let content = RT.block_on(controller.content())
		.jexcept(&mut env);
	env.new_string(content)
		.jexcept(&mut env)
		.as_raw()
}

/// Tries to fetch a [crate::api::TextChange], or returns null if there's nothing.
#[no_mangle]
pub extern "system" fn Java_mp_code_BufferController_try_1recv(
	mut env: JNIEnv,
	_class: JClass,
	self_ptr: jlong,
) -> jobject {
	let controller = unsafe { Box::leak(Box::from_raw(self_ptr as *mut crate::buffer::Controller)) };
	let change = controller.try_recv().jexcept(&mut env);
	recv_jni(&mut env, change)
}

/// Blocks until it receives a [crate::api::TextChange].
#[no_mangle]
pub extern "system" fn Java_mp_code_BufferController_recv(
	mut env: JNIEnv,
	_class: JClass,
	self_ptr: jlong,
) -> jobject {
	let controller = unsafe { Box::leak(Box::from_raw(self_ptr as *mut crate::buffer::Controller)) };
	let change = RT.block_on(controller.recv()).map(Some).jexcept(&mut env);
	recv_jni(&mut env, change)
}

/// Utility method to convert a [crate::api::TextChange] to its Java equivalent.
fn recv_jni(env: &mut JNIEnv, change: Option<crate::api::TextChange>) -> jobject {
	match change {
		None => JObject::default(),
		Some(event) => {
			let content = env.new_string(event.content).jexcept(env);
			env.find_class("mp/code/data/TextChange")
				.and_then(|class| {
					env.new_object(
						class,
						"(JJLjava/lang/String;)V",
						&[
							JValueGen::Long(jlong::from(event.start)),
							JValueGen::Long(jlong::from(event.end)),
							JValueGen::Object(&content),
						]
					)
				}).jexcept(env)
		}
	}.as_raw()
}

/// Receives from Java, converts and sends a [crate::api::TextChange].
#[no_mangle]
pub extern "system" fn Java_mp_code_BufferController_send<'local>(
	mut env: JNIEnv,
	_class: JClass<'local>,
	self_ptr: jlong,
	input: JObject<'local>
) {
	let start = env.get_field(&input, "start", "J").and_then(|s| s.j()).jexcept(&mut env);
	let end = env.get_field(&input, "end", "J").and_then(|e| e.j()).jexcept(&mut env);
	let content = env.get_field(&input, "content", "Ljava/lang/String;")
		.and_then(|c| c.l())
		.map(|c| c.into())
		.jexcept(&mut env);
	let content = unsafe { env.get_string_unchecked(&content) }
		.map(|c| c.to_string_lossy().to_string())
		.jexcept(&mut env);

	let controller = unsafe { Box::leak(Box::from_raw(self_ptr as *mut crate::buffer::Controller)) };
	controller.send(crate::api::TextChange {
		start: start as u32,
		end: end as u32,
		content
	}).jexcept(&mut env);
}

/// Called by the Java GC to drop a [crate::buffer::Controller].
#[no_mangle]
pub extern "system" fn Java_mp_code_BufferController_free(
	_env: JNIEnv,
	_class: JClass,
	self_ptr: jlong,
) {
	let _ = unsafe { Box::from_raw(self_ptr as *mut crate::cursor::Controller) };
}

