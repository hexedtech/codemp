use jni_toolbox::jni;

use crate::{
	errors::ControllerError,
	prelude::{
		CodempAsyncReceiver as AsyncReceiver, CodempAsyncSender as AsyncSender,
		CodempCursorEvent as CursorEvent, CodempCursorUpdate as CursorUpdate,
		CodempWorkspaceIdentifier as WorkspaceIdentifier,
	},
};

/// Get the [WorkspaceIdentifier] of the workspace that contains this buffer.
#[jni(package = "mp.code", class = "CursorController")]
fn workspace_id(controller: &mut crate::cursor::Controller) -> WorkspaceIdentifier {
	controller.workspace_id().clone()
}

/// Try to fetch a [Cursor], or returns null if there's nothing.
#[jni(package = "mp.code", class = "CursorController")]
fn try_recv(
	controller: &mut crate::cursor::Controller,
) -> Result<Option<CursorEvent>, ControllerError> {
	super::tokio().block_on(controller.try_recv())
}

/// Block until it receives a [Cursor].
#[jni(package = "mp.code", class = "CursorController")]
fn recv(controller: &mut crate::cursor::Controller) -> Result<CursorEvent, ControllerError> {
	super::tokio().block_on(controller.recv())
}

/// Receive from Java, converts and sends a [Cursor].
#[jni(package = "mp.code", class = "CursorController")]
fn send(
	controller: &mut crate::cursor::Controller,
	sel: CursorUpdate,
) -> Result<(), ControllerError> {
	controller.send(sel)
}

/// Register a callback for cursor changes.
#[jni(package = "mp.code", class = "CursorController")]
fn callback<'local>(
	env: &mut jni::Env<'local>,
	controller: &mut crate::cursor::Controller,
	cb: jni::objects::JObject<'local>,
) -> Result<(), jni::errors::Error> {
	if cb.is_null() {
		return Err(jni::errors::Error::NullPtr("cursor callback is null"));
	}

	let cb_ref = env.new_global_ref(cb)?;
	let jvm = env.get_java_vm()?;

	controller.callback(move |controller: crate::cursor::Controller| {
		let res: Result<(), jni::errors::Error> = jvm.attach_current_thread(|env| {
			env.with_local_frame(5, |env| {
				use jni_toolbox::IntoJavaObject;
				let jcontroller = controller.into_java_object(env)?;
				env.call_method(
					&cb_ref,
					jni::jni_str!("accept"),
					jni::jni_sig!((arg1: java.lang.Object) -> ()),
					&[jni::objects::JValue::Object(&jcontroller)],
				)?;
				Ok::<(), jni::errors::Error>(())
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
#[allow(unsafe_code)]
#[jni(package = "mp.code", class = "CursorController")]
fn free(input: jni::sys::jlong) {
	let _ = unsafe { Box::from_raw(input as *mut crate::cursor::Controller) };
}
