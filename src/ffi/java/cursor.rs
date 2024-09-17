use jni::{objects::{JClass, JObject}, sys::{jboolean, jlong, jobject}, JNIEnv};
use crate::api::{Controller, Cursor};

use super::{handle_error, null_check, tokio, Deobjectify, JExceptable, JObjectify};

/// Try to fetch a [Cursor], or returns null if there's nothing.
#[no_mangle]
pub extern "system" fn Java_mp_code_CursorController_try_1recv(
	mut env: JNIEnv,
	_class: JClass,
	self_ptr: jlong,
) -> jobject {
	let controller = unsafe { Box::leak(Box::from_raw(self_ptr as *mut crate::cursor::Controller)) };
	tokio().block_on(controller.try_recv())
		.jexcept(&mut env)
		.map(|change| change.jobjectify(&mut env).jexcept(&mut env).as_raw())
		.unwrap_or_else(std::ptr::null_mut)
}

/// Block until it receives a [Cursor].
#[no_mangle]
pub extern "system" fn Java_mp_code_CursorController_recv(
	mut env: JNIEnv,
	_class: JClass,
	self_ptr: jlong,
) -> jobject {
	let controller = unsafe { Box::leak(Box::from_raw(self_ptr as *mut crate::cursor::Controller)) };
	tokio().block_on(controller.recv())
		.jexcept(&mut env)
		.jobjectify(&mut env)
		.jexcept(&mut env)
		.as_raw()
}

/// Receive from Java, converts and sends a [Cursor].
#[no_mangle]
pub extern "system" fn Java_mp_code_CursorController_send<'local>(
	mut env: JNIEnv<'local>,
	_class: JClass<'local>,
	self_ptr: jlong,
	cursor: JObject<'local>,
) {
	null_check!(env, cursor, {});
	let controller = unsafe { Box::leak(Box::from_raw(self_ptr as *mut crate::cursor::Controller)) };
	let cursor = Cursor::deobjectify(&mut env, cursor);
	if let Ok(cursor) = cursor {
		tokio().block_on(controller.send(cursor)).jexcept(&mut env)
	} else {
		handle_error!(&mut env, cursor, {});
	}
}

/// Registers a callback for cursor changes.
#[no_mangle]
pub extern "system" fn Java_mp_code_CursorController_callback<'local>(
	mut env: JNIEnv,
	_class: JClass<'local>,
	self_ptr: jlong,
	cb: JObject<'local>,
) {
	null_check!(env, cb, {});
	let controller = unsafe { Box::leak(Box::from_raw(self_ptr as *mut crate::cursor::Controller)) };
	
	let Ok(cb_ref) = env.new_global_ref(cb) else {
		env.throw_new("mp/code/exceptions/JNIException", "Failed to pin callback reference!")
			.expect("Failed to throw exception!");
		return;
	};

	controller.callback(move |controller: crate::cursor::Controller| {
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

/// Clears the callback for cursor changes.
#[no_mangle]
pub extern "system" fn Java_mp_code_CursorController_clear_1callback(
	_env: JNIEnv,
	_class: JClass,
	self_ptr: jlong,
) {
	unsafe { Box::leak(Box::from_raw(self_ptr as *mut crate::cursor::Controller)) }
		.clear_callback();
}

/// Blocks until there is a new value available.
#[no_mangle]
pub extern "system" fn Java_mp_code_CursorController_poll(
	mut env: JNIEnv,
	_class: JClass,
	self_ptr: jlong,
) {
	let controller = unsafe { Box::leak(Box::from_raw(self_ptr as *mut crate::cursor::Controller)) };
	tokio().block_on(controller.poll())
		.jexcept(&mut env);
}

/// Stops the controller.
#[no_mangle]
pub extern "system" fn Java_mp_code_CursorController_stop(
	_env: JNIEnv,
	_class: JClass,
	self_ptr: jlong,
) -> jboolean {
	let controller = unsafe { Box::leak(Box::from_raw(self_ptr as *mut crate::cursor::Controller)) };
	controller.stop() as jboolean
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
