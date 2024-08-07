use jni::{objects::{JClass, JObject, JString, JValueGen}, sys::{jlong, jobject}, JNIEnv};
use crate::{client::Client, Workspace};

use super::{util::JExceptable, RT};

/// Connects to a given URL and returns a [Client] to interact with that server.
#[no_mangle]
pub extern "system" fn Java_mp_code_Client_connect<'local>(
	mut env: JNIEnv,
	_class: JClass<'local>,
	input: JString<'local>
) -> jobject {
	let url: String = env.get_string(&input).expect("Couldn't get java string!").into();
	RT.block_on(crate::Client::new(&url))
		.map(|client| Box::into_raw(Box::new(client)) as jlong)
		.map(|ptr| {
			let class = env.find_class("mp/code/Client").expect("Failed to find class");
			env.new_object(class, "(J)V", &[JValueGen::Long(ptr)])
			.expect("Failed to initialise object")
		}).jexcept(&mut env).as_raw()
}

/// Logs in to a specific [Workspace].
#[no_mangle]
pub extern "system" fn Java_mp_code_Client_login<'local>(
	mut env: JNIEnv<'local>,
	_class: JClass<'local>,
	self_ptr: jlong,
	user: JString<'local>,
	pwd: JString<'local>,
	workspace: JString<'local>
) {
	let client = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Client)) };
	let user: String = env.get_string(&user).expect("Couldn't get java string!").into();
	let pwd: String = env.get_string(&pwd).expect("Couldn't get java string!").into();
	let workspace: String = env.get_string(&workspace).expect("Couldn't get java string!").into();
	RT.block_on(client.login(user, pwd, Some(workspace)))
		.jexcept(&mut env)
}

/// Joins a [Workspace] and returns a pointer to it.
#[no_mangle]
pub extern "system" fn Java_mp_code_Client_join_1workspace<'local>(
	mut env: JNIEnv<'local>,
	_class: JClass<'local>,
	self_ptr: jlong,
	input: JString<'local>
) -> jobject {
	let client  = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Client)) };
	let workspace_id = unsafe { env.get_string_unchecked(&input).expect("Couldn't get java string!") };
	RT.block_on(client.join_workspace(workspace_id.to_str().expect("Not UTF-8")))
		.map(|workspace| spawn_updater(workspace.clone()))
		.map(|workspace| Box::into_raw(Box::new(workspace)) as jlong)
		.map(|ptr| {
			let class = env.find_class("mp/code/Workspace").expect("Failed to find class");
			env.new_object(class, "(J)V", &[JValueGen::Long(ptr)])
			.expect("Failed to initialise object")
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

/// Gets a [Workspace] by name and returns a pointer to it.
#[no_mangle]
pub extern "system" fn Java_mp_code_Client_get_1workspace<'local>(
	mut env: JNIEnv<'local>,
	_class: JClass<'local>,
	self_ptr: jlong,
	input: JString<'local>
) -> jobject {
	let client  = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Client)) };
	let workspace_id = unsafe { env.get_string_unchecked(&input).expect("Couldn't get java string!") };
	client.get_workspace(workspace_id.to_str().expect("Not UTF-8"))
		.map(|workspace| Box::into_raw(Box::new(workspace)) as jlong)
		.map(|ptr| {
			let class = env.find_class("mp/code/Workspace").expect("Failed to find class");
			env.new_object(class, "(J)V", &[JValueGen::Long(ptr)])
			.expect("Failed to initialise object")
		}).unwrap_or(JObject::null()).as_raw()
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
			.map(|p| env.get_string(&p).expect("couldn't get java string").into())
	);
}

/// Called by the Java GC to drop a [Client].
#[no_mangle]
pub extern "system" fn Java_mp_code_Client_free(_env: JNIEnv, _class: JClass, input: jlong) {
	let _ = unsafe { Box::from_raw(input as *mut Client) };
}
