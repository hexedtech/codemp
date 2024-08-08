use jni::{objects::{JClass, JObject, JValueGen}, sys::{jlong, jobject, jstring}, JNIEnv};

use crate::api::Controller;

use super::{util::JExceptable, RT};

/// Gets the name of the buffer. 
#[no_mangle]
pub extern "system" fn Java_mp_code_BufferController_get_1name(
	env: JNIEnv,
	_class: JClass,
	self_ptr: jlong,
) -> jstring {
	let controller = unsafe { Box::leak(Box::from_raw(self_ptr as *mut crate::buffer::Controller)) };
	let content = controller.name();
	env.new_string(content)
		.expect("could not create jstring")
		.as_raw()
}

/// Gets the contents of the buffers.
#[no_mangle]
pub extern "system" fn Java_mp_code_BufferController_get_1content(
	env: JNIEnv,
	_class: JClass,
	self_ptr: jlong,
) -> jstring {
	let controller = unsafe { Box::leak(Box::from_raw(self_ptr as *mut crate::buffer::Controller)) };
	let content = controller.content();
	env.new_string(content)
		.expect("could not create jstring")
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
		None => JObject::null().as_raw(),
		Some(event) => {
			let class = env.find_class("mp/code/data/TextChange").expect("Couldn't find class!");
			env.new_object(
				class,
				"(JJLjava/lang/String;)V",
				&[
					JValueGen::Long(jlong::from(event.start)),
					JValueGen::Long(jlong::from(event.end)),
					JValueGen::Object(&env.new_string(event.content).expect("Failed to create String!")),
				]
			).expect("failed creating object").into_raw()
		}
	}
}

/// Receives from Java, converts and sends a [crate::api::TextChange].
#[no_mangle]
pub extern "system" fn Java_mp_code_BufferController_send<'local>(
	mut env: JNIEnv,
	_class: JClass<'local>,
	self_ptr: jlong,
	input: JObject<'local>
) {
	let start = env.get_field(&input, "start", "J").expect("could not get field").j().expect("field was not of expected type");
	let end = env.get_field(&input, "end", "J").expect("could not get field").j().expect("field was not of expected type");
	let content = env.get_field(&input, "content", "Ljava/lang/String;")
		.expect("could not get field")
		.l()
		.expect("field was not of expected type")
		.into();
	let content = env.get_string(&content).expect("Failed to get String!").into();

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

