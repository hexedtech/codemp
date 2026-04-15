use jni_toolbox::jni;

use crate::{
	errors::{ConnectionError, ControllerError, RemoteError},
	prelude::*
};

/// Connect using the given credentials to the default server, and return a [Client] to interact with it.
#[jni(package = "mp.code", class = "Client")]
fn connect(config: CodempConfig) -> Result<CodempClient, ConnectionError> {
	super::tokio().block_on(CodempClient::connect(config))
}

/// Gets the [UserInfo] for the current user.
#[jni(package = "mp.code", class = "Client")]
fn current_user(client: &mut CodempClient) -> CodempUserInfo {
	client.current_user().clone()
}

/// Join a [Workspace] and return a pointer to it.
#[jni(package = "mp.code", class = "Client")]
fn attach_workspace(
	client: &mut CodempClient,
	user: String,
	workspace: String,
) -> Result<CodempWorkspace, ConnectionError> {
	super::tokio().block_on(client.attach_workspace(user, workspace))
}

/// Accepts an invitation to a workspace.
#[jni(package = "mp.code", class = "Client")]
fn accept_invite(client: &mut CodempClient, user: String, workspace: String) -> Result<(), RemoteError> {
	super::tokio().block_on(client.accept_invite(user, workspace))
}

/// Rejects an invitation to a workspace.
#[jni(package = "mp.code", class = "Client")]
fn reject_invite(client: &mut CodempClient, user: String, workspace: String) -> Result<(), RemoteError> {
	super::tokio().block_on(client.reject_invite(user, workspace))
}

/// Quit a joined [Workspace].
#[jni(package = "mp.code", class = "Client")]
fn quit_workspace(client: &mut CodempClient, user: String, workspace: String) -> Result<(), RemoteError> {
	super::tokio().block_on(client.quit_workspace(user, workspace))
}

/// Create a workspace on server, if allowed to.
#[jni(package = "mp.code", class = "Client")]
fn create_workspace(client: &mut CodempClient, workspace: String) -> Result<(), RemoteError> {
	super::tokio().block_on(client.create_workspace(workspace))
}

/// Delete a workspace on server, if allowed to.
#[jni(package = "mp.code", class = "Client")]
fn delete_workspace(client: &mut CodempClient, workspace: String) -> Result<(), RemoteError> {
	super::tokio().block_on(client.delete_workspace(workspace))
}

/// Invite another user to an owned workspace.
#[jni(package = "mp.code", class = "Client")]
fn invite_to_workspace(
	client: &mut CodempClient,
	workspace: String,
	user: String,
) -> Result<(), RemoteError> {
	super::tokio().block_on(client.invite_to_workspace(workspace, user))
}

/// List owned workspaces.
#[jni(package = "mp.code", class = "Client")]
fn fetch_owned_workspaces(client: &mut CodempClient) -> Result<Vec<CodempWorkspaceIdentifier>, RemoteError> {
	super::tokio().block_on(client.fetch_owned_workspaces())
}

/// List joined workspaces.
#[jni(package = "mp.code", class = "Client")]
fn fetch_joined_workspaces(client: &mut CodempClient) -> Result<Vec<CodempWorkspaceIdentifier>, RemoteError> {
	super::tokio().block_on(client.fetch_joined_workspaces())
}

/// List available workspaces.
#[jni(package = "mp.code", class = "Client")]
fn active_workspaces(client: &mut CodempClient) -> Vec<CodempWorkspaceIdentifier> {
	client.active_workspaces()
}

/// Leave a [Workspace] and return whether or not the client was in such workspace.
#[jni(package = "mp.code", class = "Client")]
fn leave_workspace(client: &mut CodempClient, user: String, workspace: String) -> bool {
	client.leave_workspace(user, workspace)
}

/// Get a [Workspace] by name and returns a pointer to it.
#[jni(package = "mp.code", class = "Client")]
fn get_workspace(client: &mut CodempClient, user: String, workspace: String) -> Option<CodempWorkspace> {
	client.get_workspace(user, workspace)
}

/// Fetches information about a user.
#[jni(package = "mp.code", class = "Client")]
fn get_user_info(client: &mut CodempClient, user: String) -> Result<CodempUserInfo, RemoteError> {
	super::tokio().block_on(client.get_user_info(user))
}

/// Try to fetch a [TextChange], or return null if there's nothing.
#[jni(package = "mp.code", class = "Client")]
fn try_recv(client: &mut CodempClient) -> Result<Option<CodempSessionEvent>, ControllerError> {
	super::tokio().block_on(client.try_recv())
}

/// Block until it receives a [TextChange].
#[jni(package = "mp.code", class = "Client")]
fn recv(client: &mut CodempClient) -> Result<CodempSessionEvent, ControllerError> {
	super::tokio().block_on(client.recv())
}

/// Register a callback for client changes.
#[jni(package = "mp.code", class = "Client")]
fn callback<'local>(
	env: &mut jni::Env<'local>,
	client: &mut CodempClient,
	cb: jni::objects::JObject<'local>,
) -> Result<(), jni::errors::Error> {
	if cb.is_null() {
		return Err(jni::errors::Error::NullPtr(
			"null pointer to buffer callback",
		));
	}

	let cb_ref = env.new_global_ref(cb)?;
	let jvm = env.get_java_vm()?;

	client.callback(move |controller: CodempClient| {
		let result: Result<(), jni::errors::Error> = jvm.attach_current_thread(|env| {
			env.with_local_frame(5, |env| {
				use jni_toolbox::IntoJavaObject;
				let jclient = controller.into_java_object(env)?;
				env.call_method(
					&cb_ref,
					jni::jni_str!("accept"),
					jni::jni_sig!((event: java.lang.Object) -> ()),
					&[jni::objects::JValue::Object(&jclient)],
				)?;
				Ok::<(), jni::errors::Error>(())
			})?;

			Ok(())
		});

		if let Err(e) = result {
			tracing::error!("error invoking client callback: {e}");
		}
	});

	Ok(())
}

/// Clear the callback for client changes.
#[jni(package = "mp.code", class = "Client")]
fn clear_callback(client: &mut CodempClient) {
	client.clear_callback()
}

/// Block until there is a new value available.
#[jni(package = "mp.code", class = "Client")]
fn poll(client: &mut CodempClient) -> Result<(), ControllerError> {
	super::tokio().block_on(client.poll())
}

/// Refresh the client's session token.
#[jni(package = "mp.code", class = "Client")]
fn refresh(client: &mut CodempClient) -> Result<(), RemoteError> {
	super::tokio().block_on(client.refresh())
}

/// Called by the Java GC to drop a [Client].
#[allow(unsafe_code)]
#[jni(package = "mp.code", class = "Client")]
fn free(input: jni::sys::jlong) {
	let _ = unsafe { Box::from_raw(input as *mut CodempClient) };
}
