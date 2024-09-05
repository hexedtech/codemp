use jni::{objects::{JClass, JObject, JString, JValueGen}, sys::{jlong, jobject, jobjectArray, jstring}, JNIEnv};
use crate::Workspace;

use super::{JExceptable, JObjectify, RT};

/// Get the workspace id.
#[no_mangle]
pub extern "system" fn Java_mp_code_Workspace_get_1workspace_1id<'local>(
	mut env: JNIEnv<'local>,
	_class: JClass<'local>,
	self_ptr: jlong
) -> jstring {
	let workspace = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Workspace)) };
	env.new_string(workspace.id()).jexcept(&mut env).as_raw()
}

/// Get a cursor controller by name and returns a pointer to it.
#[no_mangle]
pub extern "system" fn Java_mp_code_Workspace_get_1cursor<'local>(
	mut env: JNIEnv<'local>,
	_class: JClass<'local>,
	self_ptr: jlong
) -> jobject {
	let workspace = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Workspace)) };
	env.find_class("mp/code/CursorController").and_then(|class|
		env.new_object(class, "(J)V", &[JValueGen::Long(Box::into_raw(Box::new(workspace.cursor())) as jlong)])
	).jexcept(&mut env).as_raw()
}

/// Get a buffer controller by name and returns a pointer to it.
#[no_mangle]
pub extern "system" fn Java_mp_code_Workspace_get_1buffer<'local>(
	mut env: JNIEnv<'local>,
	_class: JClass<'local>,
	self_ptr: jlong,
	input: JString<'local>
) -> jobject {
	let workspace = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Workspace)) };
	let path = unsafe { env.get_string_unchecked(&input) }
		.map(|path| path.to_string_lossy().to_string())
		.jexcept(&mut env);
	
	workspace.buffer_by_name(&path).map(|buf| {
		env.find_class("mp/code/BufferController").and_then(|class|
			env.new_object(class, "(J)V", &[JValueGen::Long(Box::into_raw(Box::new(buf)) as jlong)])
		).jexcept(&mut env)
	}).unwrap_or_default().as_raw()
}

/// Create a new buffer.
#[no_mangle]
pub extern "system" fn Java_mp_code_Workspace_create_1buffer<'local>(
	mut env: JNIEnv,
	_class: JClass<'local>,
	self_ptr: jlong,
	input: JString<'local>
) {
	let ws = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Workspace)) };
	let path = unsafe { env.get_string_unchecked(&input) }
		.map(|path| path.to_string_lossy().to_string())
		.jexcept(&mut env);
	RT.block_on(ws.create(&path))
		.jexcept(&mut env);
}

/// Get the filetree.
#[no_mangle]
pub extern "system" fn Java_mp_code_Workspace_get_1file_1tree(
	mut env: JNIEnv,
	_class: JClass,
	self_ptr: jlong,
	filter: JString 
) -> jobjectArray {
	let workspace = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Workspace)) };
	let filter: Option<String> = if filter.is_null() {
		None
	} else {
		Some(
			env.get_string(&filter)
				.map(|s| s.into())
				.jexcept(&mut env)
		)
	};

	let file_tree = workspace.filetree(filter.as_deref());
	env.find_class("java/lang/String")
		.and_then(|class| env.new_object_array(file_tree.len() as i32, class, JObject::null()))
		.map(|arr| {
			for (idx, path) in file_tree.iter().enumerate() {
				env.new_string(path)
					.and_then(|path| env.set_object_array_element(&arr, idx as i32, path))
					.jexcept(&mut env)
			}
			arr
		}).jexcept(&mut env).as_raw()
}

/// Attach to a buffer and return a pointer to its [crate::buffer::Controller].
#[no_mangle]
pub extern "system" fn Java_mp_code_Workspace_attach_1to_1buffer<'local>(
	mut env: JNIEnv,
	_class: JClass<'local>,
	self_ptr: jlong,
	input: JString<'local>
) -> jobject {
	let workspace = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Workspace)) };
	let path = unsafe { env.get_string_unchecked(&input) }
		.map(|path| path.to_string_lossy().to_string())
		.jexcept(&mut env);
	RT.block_on(workspace.attach(&path))
		.map(|buffer| Box::into_raw(Box::new(buffer)) as jlong)
		.map(|ptr| {
			env.find_class("mp/code/BufferController")
				.and_then(|class| env.new_object(class, "(J)V", &[JValueGen::Long(ptr)]))
				.jexcept(&mut env)
		}).jexcept(&mut env).as_raw()
}

/// Detach from a buffer.
#[no_mangle]
pub extern "system" fn Java_mp_code_Workspace_detach_1from_1buffer<'local>(
	mut env: JNIEnv,
	_class: JClass<'local>,
	self_ptr: jlong,
	input: JString<'local>
) -> jobject {
	let workspace = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Workspace)) };
	let path = unsafe { env.get_string_unchecked(&input) }
		.map(|path| path.to_string_lossy().to_string())
		.jexcept(&mut env);
	let name = match workspace.detach(&path) {
		crate::workspace::DetachResult::NotAttached => "NOT_ATTACHED",
		crate::workspace::DetachResult::Detaching => "DETACHED",
		crate::workspace::DetachResult::AlreadyDetached => "ALREADY_DETACHED"
	};

	env.find_class("mp/code/data/DetachResult")
		.and_then(|class| env.get_static_field(class, name, "Lmp/code/data/DetachResult;"))
		.and_then(|res| res.l())
		.jexcept(&mut env)
		.as_raw()
}

/// Update the local buffer list.
#[no_mangle]
pub extern "system" fn Java_mp_code_Workspace_fetch_1buffers(
	mut env: JNIEnv,
	_class: JClass,
	self_ptr: jlong,
) {
	let workspace = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Workspace)) };
	RT.block_on(workspace.fetch_buffers()).jexcept(&mut env);
}

/// Update the local user list.
#[no_mangle]
pub extern "system" fn Java_mp_code_Workspace_fetch_1users(
	mut env: JNIEnv,
	_class: JClass,
	self_ptr: jlong,
) {
	let workspace = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Workspace)) };
	RT.block_on(workspace.fetch_users()).jexcept(&mut env);
}

/// List users attached to a buffer.
#[no_mangle]
pub extern "system" fn Java_mp_code_Workspace_list_1buffer_1users<'local>(
	mut env: JNIEnv,
	_class: JClass<'local>,
	self_ptr: jlong,
	input: JString<'local>,
) -> jobjectArray {
	let workspace = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Workspace)) };
	let buffer = unsafe { env.get_string_unchecked(&input) }
		.map(|buffer| buffer.to_string_lossy().to_string())
		.jexcept(&mut env);
	let users = RT.block_on(workspace.list_buffer_users(&buffer))
		.jexcept(&mut env);

	env.find_class("java/util/UUID")
		.and_then(|class| env.new_object_array(users.len() as i32, &class, JObject::null()))
		.map(|arr| {
			for (idx, user) in users.iter().enumerate() {
				user.id.jobjectify(&mut env)
					.and_then(|id| env.set_object_array_element(&arr, idx as i32, id))
					.jexcept(&mut env);
			}
			arr
		}).jexcept(&mut env).as_raw()
}

/// Delete a buffer.
#[no_mangle]
pub extern "system" fn Java_mp_code_Workspace_delete_1buffer<'local>(
	mut env: JNIEnv,
	_class: JClass<'local>,
	self_ptr: jlong,
	input: JString<'local>,
) {
	let workspace = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Workspace)) };
	let buffer = unsafe { env.get_string_unchecked(&input) }
		.map(|buffer| buffer.to_string_lossy().to_string())
		.jexcept(&mut env);
	RT.block_on(workspace.delete(&buffer))
		.jexcept(&mut env);
}

/// Receive a workspace event if present.
#[no_mangle]
pub extern "system" fn Java_mp_code_Workspace_event(
	mut env: JNIEnv,
	_class: JClass,
	self_ptr: jlong
) -> jobject {
	let workspace = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Workspace)) };
	RT.block_on(workspace.event())
		.map(|event| {
			let (name, arg) = match event {
				crate::api::Event::FileTreeUpdated(arg) => ("FILE_TREE_UPDATED", env.new_string(arg).unwrap_or_default()),
				crate::api::Event::UserJoin(arg) => ("USER_JOIN", env.new_string(arg).unwrap_or_default()),
				crate::api::Event::UserLeave(arg) => ("USER_LEAVE", env.new_string(arg).unwrap_or_default()),
			};
			let event_type = env.find_class("mp/code/Workspace$Event$Type")
				.and_then(|class| env.get_static_field(class, name, "Lmp/code/Workspace/Event/Type;"))
				.and_then(|f| f.l())
				.jexcept(&mut env);
			env.find_class("mp/code/Workspace$Event").and_then(|class|
				env.new_object(
					class,
					"(Lmp/code/Workspace/Event/Type;Ljava/lang/String;)V",
					&[
						JValueGen::Object(&event_type),
						JValueGen::Object(&arg)
					]
				)
			).jexcept(&mut env)
		}).jexcept(&mut env).as_raw()
}

/// Poll a list of buffers, returning the first ready one.
#[no_mangle]
pub extern "system" fn Java_mp_code_Workspace_select_1buffer(
	mut env: JNIEnv,
	_class: JClass,
	self_ptr: jlong,
	timeout: jlong
) -> jobject {
	let workspace = unsafe { Box::leak(Box::from_raw(self_ptr as *mut Workspace)) };
	let buffers = workspace.buffer_list();
	let mut controllers = Vec::default();
	for buffer in buffers {
		if let Some(controller) = workspace.buffer_by_name(&buffer) {
			controllers.push(controller);
		}
	}

	RT.block_on(crate::ext::select_buffer(
		&controllers,
		Some(std::time::Duration::from_millis(timeout as u64)),
		&RT,
	)).jexcept(&mut env)
		.map(|buf| {
			env.find_class("mp/code/BufferController").and_then(|class|
				env.new_object(class, "(J)V", &[JValueGen::Long(Box::into_raw(Box::new(buf)) as jlong)])
			).jexcept(&mut env)
		}).unwrap_or_default().as_raw()
}

/// Called by the Java GC to drop a [Workspace].
#[no_mangle]
pub extern "system" fn Java_mp_code_Workspace_free(_env: JNIEnv, _class: JClass, input: jlong) {
	let _ = unsafe { Box::from_raw(input as *mut Workspace) };
}
