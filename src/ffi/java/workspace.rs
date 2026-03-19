use jni_toolbox::jni;

use crate::{
	errors::{ConnectionError, ControllerError, RemoteError},
	prelude::{
		CodempAsyncReceiver as AsyncReceiver, CodempBufferAttributes as BufferAttributes,
		CodempBufferController as BufferController, CodempBufferNode as BufferNode,
		CodempCursorController as CursorController, CodempUserInfo as UserInfo,
		CodempWorkspace as Workspace, CodempWorkspaceEvent as WorkspaceEvent,
		CodempWorkspaceIdentifier as WorkspaceIdentifier,
	},
};

/// Get the workspace id.
#[jni(package = "mp.code", class = "Workspace")]
fn id(workspace: &mut Workspace) -> WorkspaceIdentifier {
	workspace.id().clone()
}

/// Get a cursor controller by name and returns a pointer to it.
#[jni(package = "mp.code", class = "Workspace")]
fn cursor(workspace: &mut Workspace) -> CursorController {
	workspace.cursor()
}

/// Get a buffer controller by name and returns a pointer to it.
#[jni(package = "mp.code", class = "Workspace")]
fn get_buffer(workspace: &mut Workspace, path: String) -> Option<BufferController> {
	workspace.get_buffer(&path)
}

/// Searches for buffers matching the filter.
#[jni(package = "mp.code", class = "Workspace")]
fn search_buffers(workspace: &mut Workspace, filter: Option<String>) -> Vec<BufferNode> {
	workspace.search_buffers(filter.as_deref())
}

/// Gets a list of the active buffers.
#[jni(package = "mp.code", class = "Workspace")]
fn active_buffers(workspace: &mut Workspace) -> Vec<String> {
	workspace.active_buffers()
}

/// Gets a list of the active buffers.
#[jni(package = "mp.code", class = "Workspace")]
fn user_list(workspace: &mut Workspace) -> Vec<UserInfo> {
	workspace.user_list()
}

/// Create a new buffer.
#[jni(package = "mp.code", class = "Workspace")]
fn create_buffer(
	workspace: &mut Workspace,
	path: String,
	attributes: Option<BufferAttributes>,
) -> Result<(), RemoteError> {
	super::tokio().block_on(workspace.create_buffer(path, attributes))
}

/// Pins an ephemeral buffer.
#[jni(package = "mp.code", class = "Workspace")]
fn pin_buffer(workspace: &mut Workspace, path: String) -> Result<(), RemoteError> {
	super::tokio().block_on(workspace.pin_buffer(path))
}

/// Un-pins an ephemeral buffer.
#[jni(package = "mp.code", class = "Workspace")]
fn un_pin_buffer(workspace: &mut Workspace, path: String) -> Result<(), RemoteError> {
	super::tokio().block_on(workspace.un_pin_buffer(path))
}

/// Attach to a buffer and return a pointer to its [`BufferController`].
#[jni(package = "mp.code", class = "Workspace")]
fn attach_buffer(
	workspace: &mut Workspace,
	path: String,
) -> Result<BufferController, ConnectionError> {
	super::tokio().block_on(workspace.attach_buffer(&path))
}

/// Detach from a buffer.
#[jni(package = "mp.code", class = "Workspace")]
fn detach_buffer(workspace: &mut Workspace, path: String) -> bool {
	workspace.detach_buffer(&path)
}

/// Update the local buffer list.
#[jni(package = "mp.code", class = "Workspace")]
fn fetch_buffers(workspace: &mut Workspace) -> Result<(), RemoteError> {
	super::tokio().block_on(workspace.fetch_buffers())
}

/// Update the local user list.
#[jni(package = "mp.code", class = "Workspace")]
fn fetch_users(workspace: &mut Workspace) -> Result<(), RemoteError> {
	super::tokio().block_on(workspace.fetch_users())
}

/// Fetch users attached to a buffer.
#[jni(package = "mp.code", class = "Workspace")]
fn fetch_buffer_users(workspace: &mut Workspace, path: String) -> Result<(), RemoteError> {
	super::tokio().block_on(workspace.fetch_buffer_users(&path))
}

/// Fetch users attached to a buffer.
#[jni(package = "mp.code", class = "Workspace")]
fn buffer_user_list(workspace: &mut Workspace, path: String) -> Vec<UserInfo> {
	workspace.buffer_user_list(&path)
}

/// Delete a buffer.
#[jni(package = "mp.code", class = "Workspace")]
fn delete_buffer(workspace: &mut Workspace, path: String) -> Result<(), RemoteError> {
	super::tokio().block_on(workspace.delete_buffer(&path))
}

/// Block and receive a workspace event.
#[jni(package = "mp.code", class = "Workspace")]
fn recv(workspace: &mut Workspace) -> Result<WorkspaceEvent, ControllerError> {
	super::tokio().block_on(workspace.recv())
}

/// Receive a workspace event if present.
#[jni(package = "mp.code", class = "Workspace")]
fn try_recv(workspace: &mut Workspace) -> Result<Option<WorkspaceEvent>, ControllerError> {
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
	env: &mut jni::Env<'local>,
	controller: &mut Workspace,
	cb: jni::objects::JObject<'local>,
) -> Result<(), jni::errors::Error> {
	if cb.is_null() {
		return Err(jni::errors::Error::NullPtr(
			"null pointer to workspace callback",
		));
	}

	let cb_ref = env.new_global_ref(cb)?;
	let jvm = env.get_java_vm()?;

	controller.callback(move |workspace: Workspace| {
		let out: Result<(), jni::errors::Error> = jvm.attach_current_thread(|env| {
			env.with_local_frame(5, |env| {
				use jni_toolbox::IntoJavaObject;
				let jworkspace = workspace.into_java_object(env)?;
				env.call_method(
					&cb_ref,
					jni::jni_str!("accept"),
					jni::jni_sig!((ws: java.lang.Object) -> ()),
					&[jni::objects::JValue::Object(&jworkspace)],
				)?;
				Ok::<(), jni::errors::Error>(())
			})?;
			Ok(())
		});

		if let Err(e) = out {
			tracing::error!("error invoking workspace callback: {e}");
		}
	});

	Ok(())
}

/// Called by the Java GC to drop a [Workspace].
#[allow(unsafe_code)]
#[jni(package = "mp.code", class = "Workspace")]
fn free(input: jni::sys::jlong) {
	let _ = unsafe { Box::from_raw(input as *mut Workspace) };
}
