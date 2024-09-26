use jni::{objects::JObject, JNIEnv};
use jni_toolbox::jni;

use crate::{api::{Controller, TextChange}, errors::ControllerError};

use super::null_check;

/// Get the name of the buffer. 
#[jni(package = "mp.code", class = "BufferController")]
fn get_name(controller: &mut crate::buffer::Controller) -> String {
	controller.path().to_string() //TODO: &str is built into the newer version
}

/// Get the contents of the buffers.
#[jni(package = "mp.code", class = "BufferController")]
fn get_content(controller: &mut crate::buffer::Controller) -> Result<String, ControllerError> {
	super::tokio().block_on(controller.content())
}

/// Try to fetch a [TextChange], or return null if there's nothing.
#[jni(package = "mp.code", class = "BufferController")]
fn try_recv(controller: &mut crate::buffer::Controller) -> Result<Option<TextChange>, ControllerError> {
	super::tokio().block_on(controller.try_recv())
}

/// Block until it receives a [TextChange].
#[jni(package = "mp.code", class = "BufferController")]
fn recv(controller: &mut crate::buffer::Controller) -> Result<TextChange, ControllerError> {
	super::tokio().block_on(controller.recv())
}

/// Send a [TextChange] to the server.
#[jni(package = "mp.code", class = "BufferController")]
fn send(controller: &mut crate::buffer::Controller, change: TextChange) -> Result<(), ControllerError> {
	super::tokio().block_on(controller.send(change))
}

/// Register a callback for buffer changes.
#[jni(package = "mp.code", class = "BufferController")]
fn callback<'local>(env: &mut JNIEnv<'local>, controller: &mut crate::buffer::Controller, cb: JObject<'local>) {
	null_check!(env, cb, {});
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
			use jni_toolbox::IntoJavaObject;
			let jcontroller = controller.into_java_object(env)?;
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

/// Clear the callback for buffer changes.
#[jni(package = "mp.code", class = "BufferController")]
fn clear_callback(controller: &mut crate::buffer::Controller) {
	controller.clear_callback()
}

/// Block until there is a new value available.
#[jni(package = "mp.code", class = "BufferController")]
fn poll(controller: &mut crate::buffer::Controller) -> Result<(), ControllerError> {
	super::tokio().block_on(controller.poll())
}

/// Called by the Java GC to drop a [crate::buffer::Controller].
#[jni(package = "mp.code", class = "BufferController")]
fn free(input: jni::sys::jlong) {
	let _ = unsafe { Box::from_raw(input as *mut crate::buffer::Controller) };
}
