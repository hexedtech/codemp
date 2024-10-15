use crate::{
	api::{controller::AsyncReceiver, User},
	errors::{ConnectionError, ControllerError, RemoteError},
	ffi::java::null_check,
	Workspace,
};
use jni::{objects::JObject, JNIEnv};
use jni_toolbox::jni;

/// Get the workspace id.
#[jni(package = "mp.code", class = "Workspace")]
fn id(workspace: &mut Workspace) -> String {
	workspace.id()
}

/// Get a cursor controller by name and returns a pointer to it.
#[jni(package = "mp.code", class = "Workspace")]
fn cursor(workspace: &mut Workspace) -> crate::cursor::Controller {
	workspace.cursor()
}

/// Get a buffer controller by name and returns a pointer to it.
#[jni(package = "mp.code", class = "Workspace")]
fn get_buffer(workspace: &mut Workspace, path: String) -> Option<crate::buffer::Controller> {
	workspace.get_buffer(&path)
}

/// Searches for buffers matching the filter.
#[jni(package = "mp.code", class = "Workspace")]
fn search_buffers(workspace: &mut Workspace, filter: Option<String>) -> Vec<String> {
	workspace.search_buffers(filter.as_deref())
}

/// Gets a list of the active buffers.
#[jni(package = "mp.code", class = "Workspace")]
fn active_buffers(workspace: &mut Workspace) -> Vec<String> {
	workspace.active_buffers()
}

/// Gets a list of the active buffers.
#[jni(package = "mp.code", class = "Workspace")]
fn user_list(workspace: &mut Workspace) -> Vec<User> {
	workspace.user_list()
}

/// Create a new buffer.
#[jni(package = "mp.code", class = "Workspace")]
fn create_buffer(workspace: &mut Workspace, path: String) -> Result<(), RemoteError> {
	super::tokio().block_on(workspace.create_buffer(&path))
}

/// Attach to a buffer and return a pointer to its [`crate::buffer::Controller`].
#[jni(package = "mp.code", class = "Workspace")]
fn attach_buffer(
	workspace: &mut Workspace,
	path: String,
) -> Result<crate::buffer::Controller, ConnectionError> {
	super::tokio().block_on(workspace.attach_buffer(&path))
}

/// Detach from a buffer.
#[jni(package = "mp.code", class = "Workspace")]
fn detach_buffer(workspace: &mut Workspace, path: String) -> bool {
	workspace.detach_buffer(&path)
}

/// Update the local buffer list.
#[jni(package = "mp.code", class = "Workspace")]
fn fetch_buffers(workspace: &mut Workspace) -> Result<Vec<String>, RemoteError> {
	super::tokio().block_on(workspace.fetch_buffers())
}

/// Update the local user list.
#[jni(package = "mp.code", class = "Workspace")]
fn fetch_users(workspace: &mut Workspace) -> Result<Vec<User>, RemoteError> {
	super::tokio().block_on(workspace.fetch_users())
}

/// Fetch users attached to a buffer.
#[jni(package = "mp.code", class = "Workspace")]
fn fetch_buffer_users(
	workspace: &mut Workspace,
	path: String,
) -> Result<Vec<crate::api::User>, RemoteError> {
	super::tokio().block_on(workspace.fetch_buffer_users(&path))
}

/// Delete a buffer.
#[jni(package = "mp.code", class = "Workspace")]
fn delete_buffer(workspace: &mut Workspace, path: String) -> Result<(), RemoteError> {
	super::tokio().block_on(workspace.delete_buffer(&path))
}

/// Block and receive a workspace event.
#[jni(package = "mp.code", class = "Workspace")]
fn recv(workspace: &mut Workspace) -> Result<crate::api::Event, ControllerError> {
	super::tokio().block_on(workspace.recv())
}

/// Receive a workspace event if present.
#[jni(package = "mp.code", class = "Workspace")]
fn try_recv(workspace: &mut Workspace) -> Result<Option<crate::api::Event>, ControllerError> {
	super::tokio().block_on(workspace.try_recv())
}

/// Block until a workspace event is available.
#[jni(package = "mp.code", class = "Workspace")]
fn poll(workspace: &mut Workspace) -> Result<(), ControllerError> {
	super::tokio().block_on(workspace.poll())
}

/// Clear previously registered callback.
#[jni(package = "mp.code", class = "Workspace")]
fn clear_callback(workspace: &mut Workspace) {
	workspace.clear_callback();
}

/// Register a callback for workspace events.
#[jni(package = "mp.code", class = "Workspace")]
fn callback<'local>(
	env: &mut JNIEnv<'local>,
	controller: &mut crate::Workspace,
	cb: JObject<'local>,
) {
	null_check!(env, cb, {});
	let Ok(cb_ref) = env.new_global_ref(cb) else {
		env.throw_new(
			"mp/code/exceptions/JNIException",
			"Failed to pin callback reference!",
		)
		.expect("Failed to throw exception!");
		return;
	};

	controller.callback(move |workspace: crate::Workspace| {
		let jvm = super::jvm();
		let mut env = jvm
			.attach_current_thread_permanently()
			.expect("failed attaching to main JVM thread");
		if let Err(e) = env.with_local_frame(5, |env| {
			use jni_toolbox::IntoJavaObject;
			let jworkspace = workspace.into_java_object(env)?;
			if let Err(e) = env.call_method(
				&cb_ref,
				"accept",
				"(Ljava/lang/Object;)V",
				&[jni::objects::JValueGen::Object(&jworkspace)],
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

/// Called by the Java GC to drop a [Workspace].
#[jni(package = "mp.code", class = "Workspace")]
fn free(input: jni::sys::jlong) {
	let _ = unsafe { Box::from_raw(input as *mut crate::Workspace) };
}
