use jni_toolbox::jni;

use crate::{
	errors::ControllerError,
	prelude::*
};

/// Get the name of the buffer.
#[jni(package = "mp.code", class = "BufferController")]
fn path(controller: &mut CodempBufferController) -> String {
	controller.path().to_string()
}

/// Get the [WorkspaceIdentifier] of the workspace that contains this buffer.
#[jni(package = "mp.code", class = "BufferController")]
fn workspace_id(controller: &mut CodempBufferController) -> CodempWorkspaceIdentifier {
	controller.workspace_id().clone()
}

/// Get the contents of the buffers.
#[jni(package = "mp.code", class = "BufferController")]
fn content(controller: &mut CodempBufferController) -> Result<String, ControllerError> {
	super::tokio().block_on(controller.content())
}

/// Try to fetch a [TextChange], or return null if there's nothing.
#[jni(package = "mp.code", class = "BufferController")]
fn try_recv(controller: &mut CodempBufferController) -> Result<Option<CodempBufferUpdate>, ControllerError> {
	super::tokio().block_on(controller.try_recv())
}

/// Block until it receives a [TextChange].
#[jni(package = "mp.code", class = "BufferController")]
fn recv(controller: &mut CodempBufferController) -> Result<CodempBufferUpdate, ControllerError> {
	super::tokio().block_on(controller.recv())
}

/// Send a [TextChange] to the server.
#[jni(package = "mp.code", class = "BufferController")]
fn send(controller: &mut CodempBufferController, change: CodempTextChange) -> Result<(), ControllerError> {
	controller.send(change)
}

/// Register a callback for buffer changes.
#[jni(package = "mp.code", class = "BufferController")]
fn callback<'local>(
	env: &mut jni::Env<'local>,
	controller: &mut CodempBufferController,
	cb: jni::objects::JObject<'local>,
) -> Result<(), jni::errors::Error> {
	if cb.is_null() {
		return Err(jni::errors::Error::NullPtr(
			"null pointer to buffer callback",
		));
	}

	let cb_ref = env.new_global_ref(cb)?;
	let jvm = env.get_java_vm()?;

	controller.callback(move |controller: CodempBufferController| {
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
fn clear_callback(controller: &mut CodempBufferController) {
	controller.clear_callback()
}

/// Block until there is a new value available.
#[jni(package = "mp.code", class = "BufferController")]
fn poll(controller: &mut CodempBufferController) -> Result<(), ControllerError> {
	super::tokio().block_on(controller.poll())
}

/// Acknowledge that a change has been correctly applied.
#[jni(package = "mp.code", class = "BufferController")]
fn ack(controller: &mut CodempBufferController, version: Vec<i64>) {
	controller.ack(version)
}

/// Called by the Java GC to drop a [crate::buffer::Controller].
#[allow(unsafe_code)]
#[jni(package = "mp.code", class = "BufferController")]
fn free(input: jni::sys::jlong) {
	let _ = unsafe { Box::from_raw(input as *mut CodempBufferController) };
}
