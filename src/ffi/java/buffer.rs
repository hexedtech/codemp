use jni::{objects::{JClass, JObject}, sys::{jboolean, jlong, jobject, jstring}, JNIEnv};

use crate::api::{Controller, TextChange};

use super::{handle_error, null_check, tokio, Deobjectify, JExceptable, JObjectify};

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
	let content = tokio().block_on(controller.content())
		.jexcept(&mut env);
	env.new_string(content)
		.jexcept(&mut env)
		.as_raw()
}

/// Tries to fetch a [TextChange], or returns null if there's nothing.
#[no_mangle]
pub extern "system" fn Java_mp_code_BufferController_try_1recv(
	mut env: JNIEnv,
	_class: JClass,
	self_ptr: jlong,
) -> jobject {
	let controller = unsafe { Box::leak(Box::from_raw(self_ptr as *mut crate::buffer::Controller)) };
	tokio().block_on(controller.try_recv())
		.jexcept(&mut env)
		.map(|change| change.jobjectify(&mut env).jexcept(&mut env).as_raw())
		.unwrap_or_else(std::ptr::null_mut)
}

/// Blocks until it receives a [TextChange].
#[no_mangle]
pub extern "system" fn Java_mp_code_BufferController_recv(
	mut env: JNIEnv,
	_class: JClass,
	self_ptr: jlong,
) -> jobject {
	let controller = unsafe { Box::leak(Box::from_raw(self_ptr as *mut crate::buffer::Controller)) };
	tokio().block_on(controller.recv())
		.jexcept(&mut env)
		.jobjectify(&mut env)
		.jexcept(&mut env)
		.as_raw()
}

/// Receive from Java, converts and sends a [TextChange].
#[no_mangle]
pub extern "system" fn Java_mp_code_BufferController_send<'local>(
	mut env: JNIEnv<'local>,
	_class: JClass<'local>,
	self_ptr: jlong,
	change: JObject<'local>,
) {
	null_check!(env, change, {});
	let controller = unsafe { Box::leak(Box::from_raw(self_ptr as *mut crate::buffer::Controller)) };
	let change = TextChange::deobjectify(&mut env, change);
	if let Ok(change) = change {
		tokio().block_on(controller.send(change)).jexcept(&mut env)
	} else {
		handle_error!(&mut env, change, {});
	}
}

/// Registers a callback for buffer changes.
#[no_mangle]
pub extern "system" fn Java_mp_code_BufferController_callback<'local>(
	mut env: JNIEnv,
	_class: JClass<'local>,
	self_ptr: jlong,
	cb: JObject<'local>,
) {
	null_check!(env, cb, {});
	let controller = unsafe { Box::leak(Box::from_raw(self_ptr as *mut crate::buffer::Controller)) };
	let Ok(cb_ref) = env.new_global_ref(cb) else {
		env.throw_new("mp/code/exceptions/JNIException", "Failed to pin callback reference!")
			.expect("Failed to throw exception!");
		return;
	};

	controller.callback(move |controller: crate::buffer::Controller| {
		let jvm = super::jvm();
		let mut env = jvm.attach_current_thread_permanently()
			.expect("failed attaching to main JVM thread");
		if let Err(e) = env.with_local_frame(5, |env| {
			use crate::ffi::java::JObjectify;
			let jcontroller = controller.jobjectify(env)?;
			if let Err(e) = env.call_method(
				&cb_ref,
				"accept",
				"(Ljava/lang/Object;)V",
				&[jni::objects::JValueGen::Object(&jcontroller)]
			) {
				tracing::error!("error invoking callback: {e:?}");
			};
			Ok::<(), jni::errors::Error>(())
		}) {
			tracing::error!("error invoking callback: {e}");
			let _ = env.exception_describe();
		}
	});
}

/// Clears the callback for buffer changes.
#[no_mangle]
pub extern "system" fn Java_mp_code_BufferController_clear_1callback(
	_env: JNIEnv,
	_class: JClass,
	self_ptr: jlong,
) {
	unsafe { Box::leak(Box::from_raw(self_ptr as *mut crate::buffer::Controller)) }
		.clear_callback();
}

/// Blocks until there is a new value available.
#[no_mangle]
pub extern "system" fn Java_mp_code_BufferController_poll(
	mut env: JNIEnv,
	_class: JClass,
	self_ptr: jlong,
) {
	let controller = unsafe { Box::leak(Box::from_raw(self_ptr as *mut crate::buffer::Controller)) };
	tokio().block_on(controller.poll())
		.jexcept(&mut env);
}

/// Stops the controller.
#[no_mangle]
pub extern "system" fn Java_mp_code_BufferController_stop(
	_env: JNIEnv,
	_class: JClass,
	self_ptr: jlong,
) -> jboolean {
	let controller = unsafe { Box::leak(Box::from_raw(self_ptr as *mut crate::buffer::Controller)) };
	controller.stop() as jboolean
}

/// Called by the Java GC to drop a [crate::buffer::Controller].
#[no_mangle]
pub extern "system" fn Java_mp_code_BufferController_free(
	_env: JNIEnv,
	_class: JClass,
	self_ptr: jlong,
) {
	let _ = unsafe { Box::from_raw(self_ptr as *mut crate::buffer::Controller) };
}
