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
	let content = controller.path();
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
	let change = RT.block_on(controller.try_recv()).jexcept(&mut env);
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

			let hash = env.find_class("java/util/OptionalLong").and_then(|class| {
				if let Some(h) = event.hash {
					env.call_static_method(class, "of", "(J)Ljava/util/OptionalLong;", &[JValueGen::Long(h)])
				} else {
					env.call_static_method(class, "empty", "()Ljava/util/OptionalLong;", &[])
				}
			}).and_then(|o| o.l()).jexcept(env);
			env.find_class("mp/code/data/TextChange")
				.and_then(|class| {
					env.new_object(
						class,
						"(JJLjava/lang/String;Ljava/util/OptionalLong;)V",
						&[
							JValueGen::Long(jlong::from(event.start)),
							JValueGen::Long(jlong::from(event.end)),
							JValueGen::Object(&content),
							JValueGen::Object(&hash)
						]
					)
				}).jexcept(env)
		}
	}.as_raw()
}
