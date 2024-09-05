use napi_derive::napi;
use crate::ext::hash;


#[napi(js_name = "hash")]
pub fn js_hash(str : String) -> napi::Result<i64>{
    Ok(hash(str))
}
