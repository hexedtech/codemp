use std::io::Write;
use std::sync::Mutex;

use crate::api::controller::ControllerCallback;
use crate::api::Cursor;
use crate::prelude::*;
use crate::workspace::worker::DetachResult;
use mlua::prelude::*;
use tokio::sync::mpsc;

impl From::<CodempError> for LuaError {
	fn from(value: CodempError) -> Self {
		LuaError::WithContext {
			context: value.to_string(),
			cause: std::sync::Arc::new(LuaError::external(value)),
		}
	}
}

lazy_static::lazy_static!{
	static ref RT : tokio::runtime::Runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().expect("could not create tokio runtime");
}

fn runtime_drive_forever(_: &Lua, ():()) -> LuaResult<Driver> {
	let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
	std::thread::spawn(move || RT.block_on(async move {
		tokio::select! {
			() = std::future::pending::<()>() => {},
			_ = rx.recv() => {},
		}
	}));
	Ok(Driver(tx))
}

#[derive(Debug, Clone)]
struct Driver(tokio::sync::mpsc::UnboundedSender<()>);
impl LuaUserData for Driver {
	fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(format!("{:?}", this)));
		methods.add_method("stop", |_, this, ()| Ok(this.0.send(()).is_ok()));
	}
}


impl LuaUserData for CodempClient {
	fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
		fields.add_field_method_get("id", |_, this| Ok(this.user_id().to_string()));
	}

	fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(format!("{:?}", this)));

		// join a remote workspace and start processing cursor events
		methods.add_method("join_workspace", |_, this, (session,):(String,)|
			Ok(RT.block_on(async { this.join_workspace(&session).await })?)
		);

		methods.add_method("leave_workspace", |_, this, (session,):(String,)| {
			Ok(this.leave_workspace(&session))
		});
		
		methods.add_method("get_workspace", |_, this, (session,):(String,)| Ok(this.get_workspace(&session)));
		methods.add_method("active_workspaces", |_, this, ()| Ok(this.active_workspaces()));
	}

}


impl LuaUserData for CodempWorkspace {
	fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(format!("{:?}", this)));
		methods.add_method("create_buffer", |_, this, (name,):(String,)| {
			Ok(RT.block_on(async { this.create(&name).await })?)
		});

		methods.add_method("attach", |_, this, (name,):(String,)| {
			Ok(RT.block_on(async { this.attach(&name).await })?)
		});

		methods.add_method("detach", |_, this, (name,):(String,)| {
			Ok(matches!(this.detach(&name), DetachResult::Detaching | DetachResult::AlreadyDetached))
		});

		methods.add_method("delete_buffer", |_, this, (name,):(String,)| {
			Ok(RT.block_on(this.delete(&name))?)
		});

		methods.add_method("get_buffer", |_, this, (name,):(String,)| Ok(this.buffer_by_name(&name)));

		methods.add_method("event", |_, this, ()| Ok(RT.block_on(this.event())?));

		methods.add_method("fetch_buffers", |_, this, ()| Ok(RT.block_on(this.fetch_buffers())?));
		methods.add_method("fetch_users", |_, this, ()| Ok(RT.block_on(this.fetch_users())?));
	}

	fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
		fields.add_field_method_get("id", |_, this| Ok(this.id()));
		fields.add_field_method_get("cursor", |_, this| Ok(this.cursor()));
		fields.add_field_method_get("filetree", |_, this| Ok(this.filetree()));
		fields.add_field_method_get("active_buffers", |_, this| Ok(this.buffer_list()));
		// fields.add_field_method_get("users", |_, this| Ok(this.0.users())); // TODO
	}
}

impl LuaUserData for CodempEvent {
	fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(format!("{:?}", this)));
	}

	fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
		fields.add_field_method_get("type", |_, this| match this {
			CodempEvent::FileTreeUpdated => Ok("filetree"),
			CodempEvent::UserJoin(_) | CodempEvent::UserLeave(_) => Ok("user"),
		});
		fields.add_field_method_get("value", |_, this| match this {
			CodempEvent::FileTreeUpdated => Ok(None),
			CodempEvent::UserJoin(x) | CodempEvent::UserLeave(x) => Ok(Some(x.clone())),
		});
	}
}

impl LuaUserData for CodempCursorController {
	fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(format!("{:?}", this)));

		methods.add_method("send", |_, this, (buffer, start_row, start_col, end_row, end_col):(String, i32, i32, i32, i32)| {
			Ok(RT.block_on(this.send(CodempCursor { buffer, start: (start_row, start_col), end: (end_row, end_col), user: None }))?)
		});
		methods.add_method("try_recv", |_, this, ()| Ok(RT.block_on(this.try_recv())?));
		methods.add_method("recv", |_, this, ()| Ok(RT.block_on(this.recv())?));
		methods.add_method("poll", |_, this, ()| Ok(RT.block_on(this.poll())?));

		methods.add_method("stop", |_, this, ()| Ok(this.stop()));

		methods.add_method("clear_callback", |_, this, ()| Ok(this.clear_callback()));
		methods.add_method("callback", |_, this, (cb,):(LuaFunction,)| {
			this.callback(ControllerCallback::from(move |controller| {
				if let Err(e) = cb.call::<(CodempCursorController,), ()>((controller,)) {
					tracing::error!("error running cursor callback: {e}");
				}
			}));
			Ok(())
		});
	}
}

impl LuaUserData for Cursor {
	fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(format!("{:?}", this)));
	}

	fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
		fields.add_field_method_get("user", |_, this| Ok(this.user.map(|x| x.to_string())));
		fields.add_field_method_get("buffer", |_, this| Ok(this.buffer.clone()));
		fields.add_field_method_get("start",  |_, this| Ok(RowCol::from(this.start)));
		fields.add_field_method_get("finish", |_, this| Ok(RowCol::from(this.end)));
	}
}

#[derive(Debug, Clone, Copy)]
struct RowCol {
	row: i32,
	col: i32,
}

impl From<(i32, i32)> for RowCol {
	fn from((row, col): (i32, i32)) -> Self {
		Self { row, col }
	}
}

impl LuaUserData for RowCol {
	fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(format!("{:?}", this)));
	}

	fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
		fields.add_field_method_get("row",  |_, this| Ok(this.row));
		fields.add_field_method_get("col",  |_, this| Ok(this.col));
	}
}

impl LuaUserData for CodempBufferController {
	fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(format!("{:?}", this)));

		methods.add_method("send", |_, this, (start, end, text): (usize, usize, String)| {
			Ok(
				RT.block_on(this.send(
					CodempTextChange {
						start: start as u32,
						end: end as u32,
						content: text,
						hash: None,
					}
				))?
			)
		});

		methods.add_method("try_recv", |_, this, ()| Ok(RT.block_on(this.try_recv())?));
		methods.add_method("recv", |_, this, ()| Ok(RT.block_on(this.recv())?));
		methods.add_method("poll", |_, this, ()| Ok(RT.block_on(this.poll())?));

		methods.add_method("stop", |_, this, ()| Ok(this.stop()));

		methods.add_method("content", |_, this, ()| Ok(RT.block_on(this.content())?));

		methods.add_method("clear_callback", |_, this, ()| Ok(this.clear_callback()));
		methods.add_method("callback", |_, this, (cb,):(LuaFunction,)| {
			this.callback(ControllerCallback::from(move |controller: CodempBufferController| {
				let _c = controller.clone();
				if let Err(e) = cb.call::<(CodempBufferController,), ()>((controller,)) {
					tracing::error!("error running buffer#{} callback: {e}", _c.name());
				}
			}));
			Ok(())
		});
	}
}

impl LuaUserData for CodempTextChange {
	fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
		fields.add_field_method_get("content", |_, this| Ok(this.content.clone()));
		fields.add_field_method_get("first",   |_, this| Ok(this.start));
		fields.add_field_method_get("last",  |_, this| Ok(this.end));
		fields.add_field_method_get("hash",  |_, this| Ok(this.hash));
	}

	fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(format!("{:?}", this)));
		methods.add_method("apply", |_, this, (txt,):(String,)| Ok(this.apply(&txt)));
	}
}


// define module and exports
#[mlua::lua_module]
fn codemp_lua(lua: &Lua) -> LuaResult<LuaTable> {
	let exports = lua.create_table()?;

	// entrypoint
	exports.set("connect", lua.create_function(|_, (host, username, password):(String,String,String)|
		Ok(RT.block_on(CodempClient::new(host, username, password))?)
	)?)?;

	// utils
	exports.set("hash", lua.create_function(|_, (txt,):(String,)|
		Ok(crate::hash(txt))
	)?)?;

	// runtime
	exports.set("runtime_drive_forever", lua.create_function(runtime_drive_forever)?)?;

	// logging
	exports.set("logger", lua.create_function(logger)?)?;

	Ok(exports)
}


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
fn logger(_: &Lua, (printer, debug): (LuaValue, Option<bool>)) -> LuaResult<bool> {
	let level = if debug.unwrap_or_default() { tracing::Level::DEBUG } else {tracing::Level::INFO };
	let success = match printer {
		LuaNil
		| LuaValue::Boolean(_)
		| LuaValue::LightUserData(_)
		| LuaValue::Integer(_)
		| LuaValue::Number(_)
		| LuaValue::Table(_)
		| LuaValue::Thread(_)
		| LuaValue::UserData(_)
		| LuaValue::Error(_) => return Err(LuaError::BindError), // TODO full BadArgument type??
		LuaValue::String(path) => {
			let logfile = std::fs::File::create(path.to_string_lossy()).map_err(|e| LuaError::RuntimeError(e.to_string()))?;
			let format = tracing_subscriber::fmt::format()
				.with_level(true)
				.with_target(true)
				.with_thread_ids(true)
				.with_thread_names(true)
				.with_ansi(false)
				.with_file(false)
				.with_line_number(false)
				.with_source_location(false);
			tracing_subscriber::fmt()
				.event_format(format)
				.with_max_level(level)
				.with_writer(Mutex::new(logfile))
				.try_init()
				.is_ok()
		},
		LuaValue::Function(cb) => {
			let (tx, mut rx) = mpsc::unbounded_channel();
			let format = tracing_subscriber::fmt::format()
				.with_level(true)
				.with_target(true)
				.with_thread_ids(false)
				.with_thread_names(false)
				.with_ansi(false)
				.with_file(false)
				.with_line_number(false)
				.with_source_location(false);
			let res = tracing_subscriber::fmt()
				.event_format(format)
				.with_max_level(level)
				.with_writer(Mutex::new(LuaLoggerProducer(tx)))
				.try_init()
				.is_ok();
			if res {
				RT.spawn(async move {
					while let Some(msg) = rx.recv().await {
						let _ = cb.call::<(String,),()>((msg,));
						// if the logger fails logging who logs it?
					}
				});
			}
			res
		},
	};

	Ok(success)
}
