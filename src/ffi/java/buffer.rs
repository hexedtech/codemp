use jni::{Env, objects::JObject};
use jni_toolbox::jni;

use crate::{
	api::{AsyncReceiver, AsyncSender, BufferUpdate, TextChange},
	errors::ControllerError,
};

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
fn try_recv(
	controller: &mut crate::buffer::Controller,
) -> Result<Option<BufferUpdate>, ControllerError> {
	super::tokio().block_on(controller.try_recv())
}

/// Block until it receives a [TextChange].
#[jni(package = "mp.code", class = "BufferController")]
fn recv(controller: &mut crate::buffer::Controller) -> Result<BufferUpdate, ControllerError> {
	super::tokio().block_on(controller.recv())
}

/// Send a [TextChange] to the server.
#[jni(package = "mp.code", class = "BufferController")]
fn send(
	controller: &mut crate::buffer::Controller,
	change: TextChange,
) -> Result<(), ControllerError> {
	controller.send(change)
}

/// Register a callback for buffer changes.
#[jni(package = "mp.code", class = "BufferController")]
fn callback<'local>(
	env: &mut Env<'local>,
	controller: &mut crate::buffer::Controller,
	cb: JObject<'local>,
) -> Result<(), jni::errors::Error> {
	if cb.is_null() {
		return Err(jni::errors::Error::NullPtr("null pointer to buffer callback"));
	}

	let cb_ref = env.new_global_ref(cb)?;
	let jvm =	env.get_java_vm()?;

	controller.callback(move |controller: crate::buffer::Controller| {
		let result: Result<(), jni::errors::Error> = jvm.attach_current_thread(|env| {
			env.with_local_frame(5, |env| {
				use jni_toolbox::IntoJavaObject;
				let jcontroller = controller.into_java_object(env)?;
				env.call_method(
					&cb_ref,
					jni::jni_str!("accept"),
					jni::jni_sig!((buf: java.lang.Object) -> ()),
					&[jni::objects::JValue::Object(&jcontroller)],
				)?;
				Ok::<(), jni::errors::Error>(())
			})?;

			Ok(())
		});

		if let Err(e) = result {
			tracing::error!("error invoking buffer callback: {e}");
		}
	});

	Ok(())
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

/// Acknowledge that a change has been correctly applied.
#[jni(package = "mp.code", class = "BufferController")]
fn ack(controller: &mut crate::buffer::Controller, version: Vec<i64>) {
	controller.ack(version)
}

/// Called by the Java GC to drop a [crate::buffer::Controller].
#[allow(unsafe_code)]
#[jni(package = "mp.code", class = "BufferController")]
fn free(input: jni::sys::jlong) {
	let _ = unsafe { Box::from_raw(input as *mut crate::buffer::Controller) };
}
