use crate::{
	api::Config,
	client::Client,
	errors::{ConnectionError, RemoteError},
	Workspace,
};
use jni_toolbox::jni;

/// Connect using the given credentials to the default server, and return a [Client] to interact with it.
#[jni(package = "mp.code", class = "Client")]
fn connect(config: Config) -> Result<Client, ConnectionError> {
	super::tokio().block_on(Client::connect(config))
}

/// Gets the current [crate::api::User].
#[jni(package = "mp.code", class = "Client")]
fn current_user(client: &mut Client) -> crate::api::User {
	client.current_user().clone()
}

/// Join a [Workspace] and return a pointer to it.
#[jni(package = "mp.code", class = "Client")]
fn attach_workspace(client: &mut Client, workspace: String) -> Result<Workspace, ConnectionError> {
	super::tokio().block_on(client.attach_workspace(workspace))
}

/// Create a workspace on server, if allowed to.
#[jni(package = "mp.code", class = "Client")]
fn create_workspace(client: &mut Client, workspace: String) -> Result<(), RemoteError> {
	super::tokio().block_on(client.create_workspace(workspace))
}

/// Delete a workspace on server, if allowed to.
#[jni(package = "mp.code", class = "Client")]
fn delete_workspace(client: &mut Client, workspace: String) -> Result<(), RemoteError> {
	super::tokio().block_on(client.delete_workspace(workspace))
}

/// Invite another user to an owned workspace.
#[jni(package = "mp.code", class = "Client")]
fn invite_to_workspace(
	client: &mut Client,
	workspace: String,
	user: String,
) -> Result<(), RemoteError> {
	super::tokio().block_on(client.invite_to_workspace(workspace, user))
}

/// List owned workspaces.
#[jni(package = "mp.code", class = "Client")]
fn fetch_owned_workspaces(client: &mut Client) -> Result<Vec<String>, RemoteError> {
	super::tokio().block_on(client.fetch_owned_workspaces())
}

/// List joined workspaces.
#[jni(package = "mp.code", class = "Client")]
fn fetch_joined_workspaces(client: &mut Client) -> Result<Vec<String>, RemoteError> {
	super::tokio().block_on(client.fetch_joined_workspaces())
}

/// List available workspaces.
#[jni(package = "mp.code", class = "Client")]
fn active_workspaces(client: &mut Client) -> Vec<String> {
	client.active_workspaces()
}

/// Leave a [Workspace] and return whether or not the client was in such workspace.
#[jni(package = "mp.code", class = "Client")]
fn leave_workspace(client: &mut Client, workspace: String) -> bool {
	client.leave_workspace(&workspace)
}

/// Get a [Workspace] by name and returns a pointer to it.
#[jni(package = "mp.code", class = "Client")]
fn get_workspace(client: &mut Client, workspace: String) -> Option<Workspace> {
	client.get_workspace(&workspace)
}

/// Refresh the client's session token.
#[jni(package = "mp.code", class = "Client")]
fn refresh(client: &mut Client) -> Result<(), RemoteError> {
	super::tokio().block_on(client.refresh())
}

/// Called by the Java GC to drop a [Client].
#[jni(package = "mp.code", class = "Client")]
fn free(input: jni::sys::jlong) {
	let _ = unsafe { Box::from_raw(input as *mut Client) };
}
