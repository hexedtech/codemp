use napi_derive::napi;


#[napi(js_name = "hash")]
pub fn js_hash(data: String) -> i64 {
    crate::ext::hash(data)
}

#[napi(js_name = "version")]
pub fn js_version() -> String {
    crate::version()
}
