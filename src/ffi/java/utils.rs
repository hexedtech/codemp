use jni::{objects::{JClass, JString}, sys::jlong, JNIEnv};

use super::JExceptable;

/// Calculate the XXH3 hash for a given String.
#[no_mangle]
pub extern "system" fn Java_mp_code_Utils_hash<'local>(
	mut env: JNIEnv,
	_class: JClass<'local>,
	content: JString<'local>,
) -> jlong {
	let content: String = env.get_string(&content)
		.map(|s| s.into())
		.jexcept(&mut env);
	let hash = crate::ext::hash(content.as_bytes());
	i64::from_ne_bytes(hash.to_ne_bytes())
}
