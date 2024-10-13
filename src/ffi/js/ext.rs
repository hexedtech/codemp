use napi_derive::napi;

/// Hash function
#[napi(js_name = "hash")]
pub fn js_hash(data: String) -> i64 {
	crate::ext::hash(data)
}

/// Get the current version of the client
#[napi(js_name = "version")]
pub fn js_version() -> &'static str {
	crate::version()
}
