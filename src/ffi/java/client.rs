use jni::{objects::{JClass, JString, JValueGen}, sys::{jboolean, jlong, jobject}, JNIEnv};
use crate::{client::Client, Workspace};

use super::{JExceptable, RT};

/// Connects to a given URL and returns a [Client] to interact with that server.
#[no_mangle]
pub extern "system" fn Java_mp_code_Client_connect<'local>(
	mut env: JNIEnv,
	_class: JClass<'local>,
	url: JString<'local>,
	user: JString<'local>,
	pwd: JString<'local>
) -> jobject {
	let url: String = env.get_string(&url)
		.map(|s| s.into())
		.jexcept(&mut env);
	let user: String = env.get_string(&user)
		.map(|s| s.into())
		.jexcept(&mut env);
	let pwd: String = env.get_string(&pwd)
		.map(|s| s.into())
		.jexcept(&mut env);
	RT.block_on(crate::Client::new(&url, &user, &pwd))
		.map(|client| Box::into_raw(Box::new(client)) as jlong)
		.map(|ptr| {
			env.find_class("mp/code/Client")
				.and_then(|class| env.new_object(class, "(J)V", &[JValueGen::Long(ptr)]))
				.jexcept(&mut env)
		}).jexcept(&mut env).as_raw()
}

/// Joins a [Workspace] and returns a pointer to it.
#[no_mangle]
pub extern "system" fn Java_mp_code_Client_join_1workspace<'local>(
	mut env: JNIEnv<'local>,
	_class: JClass<'local>,
	self_ptr: jlong,
	input: JString<'local>
) -> jobject {
	let client = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Client)) };
	let workspace_id = unsafe { env.get_string_unchecked(&input) }
		.map(|wid| wid.to_string_lossy().to_string())
		.jexcept(&mut env);
	RT.block_on(client.join_workspace(workspace_id))
		.map(|workspace| spawn_updater(workspace.clone()))
		.map(|workspace| Box::into_raw(Box::new(workspace)) as jlong)
		.map(|ptr| {
			env.find_class("mp/code/Workspace")
				.and_then(|class| env.new_object(class, "(J)V", &[JValueGen::Long(ptr)]))
				.jexcept(&mut env)
		}).jexcept(&mut env).as_raw()
}

// TODO: this stays until we get rid of the arc then i'll have to find a better way
fn spawn_updater(workspace: Workspace) -> Workspace {
	let w = workspace.clone();
	RT.spawn(async move {
		loop {
			tokio::time::sleep(std::time::Duration::from_secs(60)).await;
			w.fetch_buffers().await.unwrap();
			w.fetch_users().await.unwrap();
		}
	});
	workspace
}

/// Leaves a [Workspace] and returns whether or not the client was in such workspace.
#[no_mangle]
pub extern "system" fn Java_mp_code_Client_leave_1workspace<'local>(
	mut env: JNIEnv<'local>,
	_class: JClass<'local>,
	self_ptr: jlong,
	input: JString<'local>
) -> jboolean {
	let client = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Client)) };
	unsafe { env.get_string_unchecked(&input) }
		.map(|wid| wid.to_string_lossy().to_string())
		.map(|wid| client.leave_workspace(&wid) as jboolean)
		.jexcept(&mut env)
}
/// Gets a [Workspace] by name and returns a pointer to it.
#[no_mangle]
pub extern "system" fn Java_mp_code_Client_get_1workspace<'local>(
	mut env: JNIEnv<'local>,
	_class: JClass<'local>,
	self_ptr: jlong,
	input: JString<'local>
) -> jobject {
	let client = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Client)) };
	let workspace_id = unsafe { env.get_string_unchecked(&input) }
		.map(|wid| wid.to_string_lossy().to_string())
		.jexcept(&mut env);
	client.get_workspace(&workspace_id)
		.map(|workspace| Box::into_raw(Box::new(workspace)) as jlong)
		.map(|ptr| {
			env.find_class("mp/code/Workspace")
				.and_then(|class| env.new_object(class, "(J)V", &[JValueGen::Long(ptr)]))
				.jexcept(&mut env)
		}).unwrap_or_default().as_raw()
}

/// Sets up the tracing subscriber.
#[no_mangle]
pub extern "system" fn Java_mp_code_Client_setup_1tracing<'local>(
	mut env: JNIEnv,
	_class: JClass<'local>,
	path: JString<'local>
) {
	super::setup_logger(
		true,
		Some(path)
			.filter(|p| p.is_null())
			.map(|p| env.get_string(&p).map(|s| s.into()).jexcept(&mut env))
	);
}

/// Called by the Java GC to drop a [Client].
#[no_mangle]
pub extern "system" fn Java_mp_code_Client_free(_env: JNIEnv, _class: JClass, input: jlong) {
	let _ = unsafe { Box::from_raw(input as *mut Client) };
}
