use std::{io::Write, sync::Mutex};

use mlua_codemp_patch as mlua;
use mlua::prelude::*;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
struct LuaLoggerProducer(mpsc::UnboundedSender<String>);
impl Write for LuaLoggerProducer {
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		let _ = self.0.send(String::from_utf8_lossy(buf).to_string());
		Ok(buf.len())
	}

	fn flush(&mut self) -> std::io::Result<()> { Ok(()) }
}

// TODO can we make this less verbose?
pub(crate) fn setup_tracing(_: &Lua, (printer, debug): (LuaValue, Option<bool>)) -> LuaResult<bool> {
	let level = if debug.unwrap_or_default() { tracing::Level::DEBUG } else {tracing::Level::INFO };
	let format = tracing_subscriber::fmt::format()
		.with_level(true)
		.with_target(true)
		.with_thread_ids(false)
		.with_thread_names(false)
		.with_file(false)
		.with_line_number(false)
		.with_source_location(false);

	let success = match printer {
		LuaValue::Boolean(_)
		| LuaValue::LightUserData(_)
		| LuaValue::Integer(_)
		| LuaValue::Number(_)
		| LuaValue::Table(_)
		| LuaValue::Thread(_)
		| LuaValue::UserData(_)
		| LuaValue::Error(_) => return Err(LuaError::BindError), // TODO full BadArgument type??
		LuaValue::Nil => {
			tracing_subscriber::fmt()
				.event_format(format)
				.with_max_level(level)
				.with_writer(std::sync::Mutex::new(std::io::stderr()))
				.try_init()
				.is_ok()
		},
		LuaValue::String(path) => {
			let logfile = std::fs::File::create(path.to_string_lossy()).map_err(|e| LuaError::RuntimeError(e.to_string()))?;
			tracing_subscriber::fmt()
				.event_format(format)
				.with_max_level(level)
				.with_writer(Mutex::new(logfile))
				.with_ansi(false)
				.try_init()
				.is_ok()
		},
		LuaValue::Function(cb) => {
			let (tx, mut rx) = mpsc::unbounded_channel();
			let res = tracing_subscriber::fmt()
				.event_format(format)
				.with_max_level(level)
				.with_writer(Mutex::new(LuaLoggerProducer(tx)))
				.with_ansi(false)
				.try_init()
				.is_ok();
			if res {
				super::a_sync::tokio().spawn(async move {
					while let Some(msg) = rx.recv().await {
						super::callback().invoke(cb.clone(), msg);
					}
				});
			}
			res
		},
	};

	Ok(success)
}
