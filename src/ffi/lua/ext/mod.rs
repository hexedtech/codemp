pub mod a_sync;
pub mod callback;
pub mod log;

pub(crate) use a_sync::tokio;
pub(crate) use callback::callback;

#[allow(unused)] // for now
pub(crate) fn lua_parse_uuid(uuid: &str, pos: usize, name: &str) -> mlua::Result<uuid::Uuid> {
	use std::str::FromStr;
	match uuid::Uuid::from_str(uuid) {
		Ok(x) => Ok(x),
		Err(e) => Err(mlua::Error::BadArgument {
			pos,
			name: Some(name.to_string()),
			to: Some("Uuid::from_str".to_string()),
			cause: std::sync::Arc::new(mlua::Error::FromLuaConversionError {
				from: "string",
				to: "Uuid".to_string(),
				message: Some(e.to_string()),
			}),
		}),
	}
}
