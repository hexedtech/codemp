use jni::{objects::{JClass, JObject}, sys::{jboolean, jlong, jobject, jstring}, JNIEnv};

use crate::api::Controller;

use super::{JExceptable, JObjectify};

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
	let content = super::tokio().block_on(controller.content())
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
	super::tokio().block_on(controller.try_recv())
		.jexcept(&mut env)
		.map(|change| change.jobjectify(&mut env).jexcept(&mut env).as_raw())
		.unwrap_or_else(std::ptr::null_mut)
}

/// Blocks until it receives a [crate::api::TextChange].
#[no_mangle]
pub extern "system" fn Java_mp_code_BufferController_recv(
	mut env: JNIEnv,
	_class: JClass,
	self_ptr: jlong,
) -> jobject {
	let controller = unsafe { Box::leak(Box::from_raw(self_ptr as *mut crate::buffer::Controller)) };
	super::tokio().block_on(controller.recv())
		.jexcept(&mut env)
		.jobjectify(&mut env)
		.jexcept(&mut env)
		.as_raw()
}

/// Receive from Java, converts and sends a [crate::api::TextChange].
#[no_mangle]
pub extern "system" fn Java_mp_code_BufferController_send<'local>(
	mut env: JNIEnv,
	_class: JClass<'local>,
	self_ptr: jlong,
	input: JObject<'local>,
) {
	let Ok(start) = env.get_field(&input, "start", "J")
		.and_then(|sr| sr.j())
		.jexcept(&mut env)
		.try_into()
	else {
		return env.throw_new("java/lang/IllegalArgumentException", "Start index cannot be negative!")
			.expect("Failed to throw exception!");
	};
	
	let Ok(end) = env.get_field(&input, "end", "J")
		.and_then(|er| er.j())
		.jexcept(&mut env)
		.try_into()
	else {
		return env.throw_new("java/lang/IllegalArgumentException", "End index cannot be negative!")
			.expect("Failed to throw exception!");
	};

	let content = env.get_field(&input, "content", "Ljava/lang/String;")
		.and_then(|b| b.l())
		.map(|b| b.into())
		.jexcept(&mut env);
	let content = env.get_string(&content)
		.map(|b| b.into())
		.jexcept(&mut env);

	let hash = env.get_field(&input, "hash", "Ljava/util/OptionalLong;")
		.and_then(|hash| hash.l())
		.and_then(|hash| {
			if env.call_method(&hash, "isPresent", "()Z", &[]).and_then(|r| r.z()).jexcept(&mut env) {
				env.call_method(&hash, "getAsLong", "()J", &[])
					.and_then(|r| r.j())
					.map(Some)
			} else {
				Ok(None)
			}
		}).jexcept(&mut env);

	let controller = unsafe { Box::leak(Box::from_raw(self_ptr as *mut crate::buffer::Controller)) };
	super::tokio().block_on(controller.send(crate::api::TextChange {
		start,
		end,
		content,
		hash,
	})).jexcept(&mut env);
}

/// Registers a callback for buffer changes.
#[no_mangle]
pub extern "system" fn Java_mp_code_BufferController_callback<'local>(
	mut env: JNIEnv,
	_class: JClass<'local>,
	self_ptr: jlong,
	cb: JObject<'local>,
) {
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
			let sig = format!("(L{};)V", "java/lang/Object");
			if let Err(e) = env.call_method(
				&cb_ref,
				"invoke",
				&sig,
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
	super::tokio().block_on(controller.poll())
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
