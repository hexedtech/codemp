use napi_derive::napi;


#[napi(js_name = "hash")]
pub fn js_hash(str : String) -> napi::Result<i64>{
    Ok(crate::ext::hash(str))
}

#[napi(js_name = "version")]
pub fn js_version(str : String) -> napi::Result<String>{
    Ok(crate::version())
}