use jni_toolbox::jni;

use crate::{
	errors::{ConnectionError, ControllerError, RemoteError},
	prelude::*
};

/// Connect using the given credentials to the default server, and return a [Session] to interact with it.
#[jni(package = "mp.code", class = "Session")]
fn connect(config: CodempConfig) -> Result<CodempSession, ConnectionError> {
	super::tokio().block_on(CodempSession::connect(config))
}

/// Gets the [UserInfo] for the current user.
#[jni(package = "mp.code", class = "Session")]
fn current_user(session: &mut CodempSession) -> CodempUserInfo {
	session.current_user().clone()
}

/// Join a [Workspace] and return a pointer to it.
#[jni(package = "mp.code", class = "Session")]
fn attach_workspace(
	session: &mut CodempSession,
	user: String,
	workspace: String,
) -> Result<CodempWorkspace, ConnectionError> {
	super::tokio().block_on(session.attach_workspace(user, workspace))
}

/// Accepts an invitation to a workspace.
#[jni(package = "mp.code", class = "Session")]
fn accept_invite(session: &mut CodempSession, user: String, workspace: String) -> Result<(), RemoteError> {
	super::tokio().block_on(session.accept_invite(user, workspace))
}

/// Rejects an invitation to a workspace.
#[jni(package = "mp.code", class = "Session")]
fn reject_invite(session: &mut CodempSession, user: String, workspace: String) -> Result<(), RemoteError> {
	super::tokio().block_on(session.reject_invite(user, workspace))
}

/// Quit a joined [Workspace].
#[jni(package = "mp.code", class = "Session")]
fn quit_workspace(session: &mut CodempSession, user: String, workspace: String) -> Result<(), RemoteError> {
	super::tokio().block_on(session.quit_workspace(user, workspace))
}

/// Create a workspace on server, if allowed to.
#[jni(package = "mp.code", class = "Session")]
fn create_workspace(session: &mut CodempSession, workspace: String) -> Result<(), RemoteError> {
	super::tokio().block_on(session.create_workspace(workspace))
}

/// Delete a workspace on server, if allowed to.
#[jni(package = "mp.code", class = "Session")]
fn delete_workspace(session: &mut CodempSession, workspace: String) -> Result<(), RemoteError> {
	super::tokio().block_on(session.delete_workspace(workspace))
}

/// Invite another user to an owned workspace.
#[jni(package = "mp.code", class = "Session")]
fn invite_to_workspace(
	session: &mut CodempSession,
	workspace: String,
	user: String,
) -> Result<(), RemoteError> {
	super::tokio().block_on(session.invite_to_workspace(workspace, user))
}

/// List owned workspaces.
#[jni(package = "mp.code", class = "Session")]
fn fetch_owned_workspaces(session: &mut CodempSession) -> Result<Vec<CodempWorkspaceIdentifier>, RemoteError> {
	super::tokio().block_on(session.fetch_owned_workspaces())
}

/// List joined workspaces.
#[jni(package = "mp.code", class = "Session")]
fn fetch_joined_workspaces(session: &mut CodempSession) -> Result<Vec<CodempWorkspaceIdentifier>, RemoteError> {
	super::tokio().block_on(session.fetch_joined_workspaces())
}

/// List available workspaces.
#[jni(package = "mp.code", class = "Session")]
fn active_workspaces(session: &mut CodempSession) -> Vec<CodempWorkspaceIdentifier> {
	session.active_workspaces()
}

/// Leave a [Workspace] and return whether or not the session was in such workspace.
#[jni(package = "mp.code", class = "Session")]
fn leave_workspace(session: &mut CodempSession, user: String, workspace: String) -> bool {
	session.leave_workspace(user, workspace)
}

/// Get a [Workspace] by name and returns a pointer to it.
#[jni(package = "mp.code", class = "Session")]
fn get_workspace(session: &mut CodempSession, user: String, workspace: String) -> Option<CodempWorkspace> {
	session.get_workspace(user, workspace)
}

/// Fetches information about a user.
#[jni(package = "mp.code", class = "Session")]
fn get_user_info(session: &mut CodempSession, user: String) -> Result<CodempUserInfo, RemoteError> {
	super::tokio().block_on(session.get_user_info(user))
}

/// Try to fetch a [TextChange], or return null if there's nothing.
#[jni(package = "mp.code", class = "Session")]
fn try_recv(session: &mut CodempSession) -> Result<Option<CodempSessionEvent>, ControllerError> {
	super::tokio().block_on(session.try_recv())
}

/// Block until it receives a [TextChange].
#[jni(package = "mp.code", class = "Session")]
fn recv(session: &mut CodempSession) -> Result<CodempSessionEvent, ControllerError> {
	super::tokio().block_on(session.recv())
}

/// Register a callback for session changes.
#[jni(package = "mp.code", class = "Session")]
fn callback<'local>(
	env: &mut jni::Env<'local>,
	session: &mut CodempSession,
	cb: jni::objects::JObject<'local>,
) -> Result<(), jni::errors::Error> {
	if cb.is_null() {
		return Err(jni::errors::Error::NullPtr(
			"null pointer to buffer callback",
		));
	}

	let cb_ref = env.new_global_ref(cb)?;
	let jvm = env.get_java_vm()?;

	session.callback(move |controller: CodempSession| {
		let result: Result<(), jni::errors::Error> = jvm.attach_current_thread(|env| {
			env.with_local_frame(5, |env| {
				use jni_toolbox::IntoJavaObject;
				let jsession = controller.into_java_object(env)?;
				env.call_method(
					&cb_ref,
					jni::jni_str!("accept"),
					jni::jni_sig!((event: java.lang.Object) -> ()),
					&[jni::objects::JValue::Object(&jsession)],
				)?;
				Ok::<(), jni::errors::Error>(())
			})?;

			Ok(())
		});

		if let Err(e) = result {
			tracing::error!("error invoking session callback: {e}");
		}
	});

	Ok(())
}

/// Clear the callback for session changes.
#[jni(package = "mp.code", class = "Session")]
fn clear_callback(session: &mut CodempSession) {
	session.clear_callback()
}

/// Block until there is a new value available.
#[jni(package = "mp.code", class = "Session")]
fn poll(session: &mut CodempSession) -> Result<(), ControllerError> {
	super::tokio().block_on(session.poll())
}

/// Refresh the session's token.
#[jni(package = "mp.code", class = "Session")]
fn refresh(session: &mut CodempSession) -> Result<(), RemoteError> {
	super::tokio().block_on(session.refresh())
}

/// Called by the Java GC to drop a [`CodempSession`].
#[allow(unsafe_code)]
#[jni(package = "mp.code", class = "Session")]
fn free(input: jni::sys::jlong) {
	let _ = unsafe { Box::from_raw(input as *mut CodempSession) };
}
