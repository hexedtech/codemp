use jni::{objects::{JClass, JString}, sys::{jboolean, jlong}, JNIEnv};

use super::{JExceptable, null_check};

/// Calculate the XXH3 hash for a given String.
#[no_mangle]
pub extern "system" fn Java_mp_code_Extensions_hash<'local>(
	mut env: JNIEnv,
	_class: JClass<'local>,
	content: JString<'local>,
) -> jlong {
	null_check!(env, content, 0 as jlong);
	let content: String = env.get_string(&content)
		.map(|s| s.into())
		.jexcept(&mut env);
	let hash = crate::ext::hash(content.as_bytes());
	i64::from_ne_bytes(hash.to_ne_bytes())
}

/// Tells the [tokio] runtime how to drive the event loop.
#[no_mangle]
pub extern "system" fn Java_mp_code_Extensions_drive(
	_env: JNIEnv,
	_class: JClass,
	block: jboolean
) {
	if block != 0 {
		super::tokio().block_on(std::future::pending::<()>());
	} else {
		std::thread::spawn(|| {
			super::tokio().block_on(std::future::pending::<()>());
		});
	}
}

/// Set up the tracing subscriber.
#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_mp_code_Extensions_setupTracing<'local>(
	mut env: JNIEnv,
	_class: JClass<'local>,
	path: JString<'local>,
	debug: jboolean
) {
	super::setup_logger(
		debug != 0,
		Some(path)
			.filter(|p| !p.is_null())
			.map(|p| env.get_string(&p).map(|s| s.into())
			.jexcept(&mut env))
	);
}
