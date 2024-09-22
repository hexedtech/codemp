use jni::{objects::{JClass, JObject, JString}, sys::{jboolean, jlong, jobject, jobjectArray}, JNIEnv};
use jni_toolbox::{jni, FromJava, IntoJava, JniToolboxError};
use crate::{api::Config, client::Client, errors::{ConnectionError, RemoteError}, ffi::java::{handle_error, null_check}, Workspace};

use super::{Deobjectify, JExceptable, JObjectify, tokio};

impl<'j> IntoJava<'j> for Client {
	type T = jobject;

	fn into_java(self, env: &mut jni::JNIEnv<'j>) -> Result<Self::T, jni::errors::Error> {
		// Ok(Box::into_raw(Box::new(self)))
		todo!()
	}
}

impl<'j> FromJava<'j> for Client {
	type T = jobject;

	fn from_java(env: &mut jni::JNIEnv<'j>, value: Self::T) -> Result<Self, jni::errors::Error> {
		let x = unsafe { Box::leak(Box::from_raw(value as *mut Client)) };
		todo!();
		Ok(x.clone())
	}
}

impl<'j> FromJava<'j> for Config {
	type T = JObject<'j>;
	fn from_java(env: &mut jni::JNIEnv<'j>, value: Self::T) -> Result<Self, jni::errors::Error> {
		Ok(Config::deobjectify(env, value)?)
	}
}

impl<'j> IntoJava<'j> for crate::api::User {
	type T = jobject;
	fn into_java(self, env: &mut jni::JNIEnv<'j>) -> Result<Self::T, jni::errors::Error> {
		Ok(self.jobjectify(env)?.into_raw())
	}
}

impl<'j> IntoJava<'j> for Workspace {
	type T = jobject;
	fn into_java(self, env: &mut jni::JNIEnv<'j>) -> Result<Self::T, jni::errors::Error> {
		Ok(self.jobjectify(env)?.into_raw())
	}
}

impl JniToolboxError for ConnectionError {
	fn jclass(&self) -> String { // TODO pick class based on underlying type
		"mp/code/exceptions/ConnectionRemoteException".to_string()
	}
}

impl JniToolboxError for RemoteError {
	fn jclass(&self) -> String { // TODO pick class based on underlying type
		"mp/code/exceptions/ConnectionRemoteException".to_string()
	}
}

#[jni(package = "mp.code", class = "Client", ptr)]
fn connect(config: Config) -> Result<Client, ConnectionError> {
	tokio().block_on(Client::connect(config))
}

fn asd(arg: String) -> Result<Vec<String>, String> {
	Ok(arg.split('/').map(|x| x.to_string()).collect())
}

/// Gets the current [crate::api::User].
#[jni(package = "mp.code", class = "Client", ptr)]
fn get_user(client: Client) -> crate::api::User {
	client.user().clone()
}

/// Join a [Workspace] and return a pointer to it.
#[jni(package = "mp.code", class = "Client", ptr)]
fn join_workspace(client: Client, workspace: String) -> Result<Workspace, ConnectionError> {
	tokio().block_on(client.join_workspace(workspace))
}

#[jni(package = "mp.code", class = "Client")]
fn create_workspace(client: Client, workspace: String) -> Result<(), RemoteError> {
	tokio().block_on(client.create_workspace(workspace))
}

/// Delete a workspace on server, if allowed to.
#[jni(package = "mp.code", class = "Client")]
fn delete_workspace(client: Client, workspace: String) -> Result<(), RemoteError> {
	tokio().block_on(client.delete_workspace(workspace))
}

/// Invite another user to an owned workspace.
#[jni(package = "mp.code", class = "Client")]
fn invite_to_workspace(client: Client, workspace: String, user: String) -> Result<(), RemoteError> {
	tokio().block_on(client.invite_to_workspace(workspace, user))
}

/// List available workspaces.
#[jni(package = "mp.code", class = "Client", ptr)]
fn list_workspaces(client: Client, owned: bool, invited: bool) -> Result<Vec<String>, RemoteError> {
	tokio().block_on(client.list_workspaces(owned, invited))
}

/// List available workspaces.
#[jni(package = "mp.code", class = "Client", ptr)]
fn active_workspaces(client: Client) -> Vec<String> {
	client.active_workspaces()
}

/// Leave a [Workspace] and return whether or not the client was in such workspace.
#[jni(package = "mp.code", class = "Client")]
fn leave_workspace(client: Client, workspace: String) -> bool {
		client.leave_workspace(&workspace)
}

/// Get a [Workspace] by name and returns a pointer to it.
#[jni(package = "mp.code", class = "Client", ptr)]
fn get_workspace(client: Client, workspace: String) -> Option<Workspace> {
	client.get_workspace(&workspace)
}

/// Refresh the client's session token.
#[jni(package = "mp.code", class = "Client")]
fn refresh(client: Client) -> Result<(), RemoteError> {
	tokio().block_on(client.refresh())
}

/// Called by the Java GC to drop a [Client].
#[no_mangle]
pub extern "system" fn Java_mp_code_Client_free(_env: JNIEnv, _class: JClass, input: jlong) {
	let _ = unsafe { Box::from_raw(input as *mut Client) };
}

// TODO: this stays until we get rid of the arc then i'll have to find a better way
fn spawn_updater(workspace: Workspace) -> Workspace {
	let w = workspace.clone();
	tokio().spawn(async move {
		loop {
			tokio::time::sleep(std::time::Duration::from_secs(60)).await;
			w.fetch_buffers().await.unwrap();
			w.fetch_users().await.unwrap();
		}
	});
	workspace
}
