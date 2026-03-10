use crate::{
	api::{AsyncReceiver, AsyncSender, Cursor, CursorEvent},
	errors::ControllerError,
};
use jni::{Env, objects::JObject};
use jni_toolbox::jni;

use super::null_check;

/// Try to fetch a [Cursor], or returns null if there's nothing.
#[jni(package = "mp.code", class = "CursorController")]
fn try_recv(controller: &mut crate::cursor::Controller) -> Result<Option<CursorEvent>, ControllerError> {
	super::tokio().block_on(controller.try_recv())
}

/// Block until it receives a [Cursor].
#[jni(package = "mp.code", class = "CursorController")]
fn recv(controller: &mut crate::cursor::Controller) -> Result<CursorEvent, ControllerError> {
	super::tokio().block_on(controller.recv())
}

/// Receive from Java, converts and sends a [Cursor].
#[jni(package = "mp.code", class = "CursorController")]
fn send(controller: &mut crate::cursor::Controller, sel: Cursor) -> Result<(), ControllerError> {
	controller.send(sel)
}

/// Register a callback for cursor changes.
#[jni(package = "mp.code", class = "CursorController")]
fn callback<'local>(
	env: &mut Env<'local>,
	controller: &mut crate::cursor::Controller,
	cb: JObject<'local>,
) -> Result<(), jni::errors::Error> {
	null_check!(cb);
	let cb_ref = env.new_global_ref(cb)?;

	controller.callback(move |controller: crate::cursor::Controller| {
		let res = super::jvm().attach_current_thread(|mut env| {
			env.with_local_frame(5, |env| {
				use jni_toolbox::IntoJavaObject;
				let jcontroller = controller.into_java_object(env)?;
				env.call_method(
					&cb_ref,
					jni::jni_str!("accept"),
					jni::jni_sig!((arg1: java.lang.Object) -> ()),
					&[jni::objects::JValue::Object(&jcontroller)],
				)?;
				Ok(())
			})?;

			Ok(())
		});

		if let Err(e) = res {
			tracing::error!("error invoking cursor callback: {e}");
		}
	});

	Ok(())
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
