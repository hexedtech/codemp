use jni_toolbox::jni;
use crate::{errors::{ConnectionError, ControllerError, RemoteError}, Workspace};

/// Get the workspace id.
#[jni(package = "mp.code", class = "Workspace")]
fn get_workspace_id(workspace: &mut Workspace) -> String {
	workspace.id()
}

/// Get a cursor controller by name and returns a pointer to it.
#[jni(package = "mp.code", class = "Workspace")]
fn get_cursor(workspace: &mut Workspace) -> crate::cursor::Controller {
	workspace.cursor()
}

/// Get a buffer controller by name and returns a pointer to it.
#[jni(package = "mp.code", class = "Workspace")]
fn get_buffer(workspace: &mut Workspace, path: String) -> Option<crate::buffer::Controller> {
	workspace.buffer_by_name(&path)
}

/// Get the filetree.
#[jni(package = "mp.code", class = "Workspace")]
fn get_file_tree(workspace: &mut Workspace, filter: Option<String>, strict: bool) -> Vec<String> {
	workspace.filetree(filter.as_deref(), strict)
}

/// Gets a list of the active buffers.
#[jni(package = "mp.code", class = "Workspace")]
fn active_buffers(workspace: &mut Workspace) -> Vec<String> {
	workspace.buffer_list()
}

/// Gets a list of the active buffers.
#[jni(package = "mp.code", class = "Workspace")]
fn user_list(workspace: &mut Workspace) -> Vec<String> {
	workspace.user_list()
}

/// Create a new buffer.
#[jni(package = "mp.code", class = "Workspace")]
fn create_buffer(workspace: &mut Workspace, path: String) -> Result<(), RemoteError> {
	super::tokio().block_on(workspace.create(&path))
}

/// Attach to a buffer and return a pointer to its [crate::buffer::Controller].
#[jni(package = "mp.code", class = "Workspace")]
fn attach_to_buffer(workspace: &mut Workspace, path: String) -> Result<crate::buffer::Controller, ConnectionError> {
	super::tokio().block_on(workspace.attach(&path))
}

/// Detach from a buffer.
#[jni(package = "mp.code", class = "Workspace")]
fn detach_from_buffer(workspace: &mut Workspace, path: String) -> bool {
	workspace.detach(&path)
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

/// List users attached to a buffer.
#[jni(package = "mp.code", class = "Workspace")]
fn list_buffer_users(workspace: &mut Workspace, path: String) -> Result<Vec<crate::api::User>, RemoteError> {
	super::tokio().block_on(workspace.list_buffer_users(&path))
}

/// Delete a buffer.
#[jni(package = "mp.code", class = "Workspace")]
fn delete_buffer(workspace: &mut Workspace, path: String) -> Result<(), RemoteError> {
	super::tokio().block_on(workspace.delete(&path))
}

/// Receive a workspace event if present.
#[jni(package = "mp.code", class = "Workspace")]
fn event(workspace: &mut Workspace) -> Result<crate::api::Event, ControllerError> {
	super::tokio().block_on(workspace.event())
}

/// Called by the Java GC to drop a [Workspace].
#[jni(package = "mp.code", class = "Workspace")]
fn free(input: jni::sys::jlong) {
	let _ = unsafe { Box::from_raw(input as *mut crate::Workspace) };
}
