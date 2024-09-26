use jni::{objects::JObject, JNIEnv};
use jni_toolbox::jni;
use crate::{api::{Controller, Cursor}, errors::ControllerError};

use super::null_check;

/// Try to fetch a [Cursor], or returns null if there's nothing.
#[jni(package = "mp.code", class = "CursorController")]
fn try_recv(controller: &mut crate::cursor::Controller) -> Result<Option<Cursor>, ControllerError> {
	super::tokio().block_on(controller.try_recv())
}

/// Block until it receives a [Cursor].
#[jni(package = "mp.code", class = "CursorController")]
fn recv(controller: &mut crate::cursor::Controller) -> Result<Cursor, ControllerError> {
	super::tokio().block_on(controller.recv())
}

/// Receive from Java, converts and sends a [Cursor].
#[jni(package = "mp.code", class = "CursorController")]
fn send(controller: &mut crate::cursor::Controller, cursor: Cursor) -> Result<(), ControllerError> {
	super::tokio().block_on(controller.send(cursor))
}

/// Register a callback for cursor changes.
#[jni(package = "mp.code", class = "CursorController")]
fn callback<'local>(env: &mut JNIEnv<'local>, controller: &mut crate::cursor::Controller, cb: JObject<'local>) {
	null_check!(env, cb, {});	
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

/// Clear the callback for cursor changes.
#[jni(package = "mp.code", class = "CursorController")]
fn clear_callback(controller: &mut crate::cursor::Controller) {
	controller.clear_callback()	
}

/// Block until there is a new value available.
#[jni(package = "mp.code", class = "CursorController")]
fn poll(controller: &mut crate::cursor::Controller) -> Result<(), ControllerError> {
	super::tokio().block_on(controller.poll())
}

/// Called by the Java GC to drop a [crate::cursor::Controller].
#[jni(package = "mp.code", class = "CursorController")]
fn free(input: jni::sys::jlong) {
	let _ = unsafe { Box::from_raw(input as *mut crate::cursor::Controller) };
}
