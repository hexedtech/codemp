use std::io::Write;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

use crate::api::Cursor;
use crate::prelude::*;
use mlua::prelude::*;
use tokio::runtime::Runtime;
use tokio::sync::broadcast;

lazy_static::lazy_static!{
	// TODO use a runtime::Builder::new_current_thread() runtime to not behave like malware
	static ref STATE : GlobalState = GlobalState::default();
	static ref LOG : broadcast::Sender<String> = broadcast::channel(32).0;
	static ref ONCE : AtomicBool = AtomicBool::new(false);
}

struct GlobalState {
	client: std::sync::RwLock<CodempClient>,
	runtime: Runtime,
}

impl Default for GlobalState {
	fn default() -> Self {
		let rt = Runtime::new().expect("could not create tokio runtime");
		let addr = std::env::var("CODEMP_SERVER_ADDRESS").unwrap_or_else(|_|"http://codemp.alemi.dev:50053".to_string());
		let client = rt.block_on(
			CodempClient::new(&addr)
		).expect("could not connect to codemp servers");
		GlobalState { client: std::sync::RwLock::new(client), runtime: rt }
	}
}

impl GlobalState {
	fn client(&self) -> std::sync::RwLockReadGuard<CodempClient> {
		self.client.read().unwrap()
	}

	fn client_mut(&self) -> std::sync::RwLockWriteGuard<CodempClient> {
		self.client.write().unwrap()
	}

	fn rt(&self) -> &Runtime {
		&self.runtime
	}
}

impl From::<CodempError> for LuaError {
	fn from(value: CodempError) -> Self {
		LuaError::external(value)
	}
}

fn id(_: &Lua, (): ()) -> LuaResult<String> {
	Ok(STATE.client().user_id().to_string())
}


/// join a remote workspace and start processing cursor events
fn join_workspace(_: &Lua, (session,): (String,)) -> LuaResult<CodempCursorController> {
	tracing::info!("joining workspace {}", session);
	let ws = STATE.rt().block_on(async { STATE.client_mut().join_workspace(&session).await })?;
	let cursor = ws.cursor();
	Ok(cursor.into())
}

fn login(_: &Lua, (username, password, workspace_id):(String, String, String)) -> LuaResult<()> {
	Ok(STATE.rt().block_on(STATE.client().login(username, password, Some(workspace_id)))?)
}

fn get_workspace(_: &Lua, (session,): (String,)) -> LuaResult<Option<CodempWorkspace>> {
	Ok(STATE.client().get_workspace(&session))
}

impl LuaUserData for CodempWorkspace {
	fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_method("create_buffer", |_, this, (name,):(String,)| {
			Ok(STATE.rt().block_on(async { this.create(&name).await })?)
		});

		methods.add_method("attach_buffer", |_, this, (name,):(String,)| {
			Ok(STATE.rt().block_on(async { this.attach(&name).await })?)
		});

		// TODO disconnect_buffer
		// TODO leave_workspace:w

		methods.add_method("get_buffer", |_, this, (name,):(String,)| Ok(this.buffer_by_name(&name)));
	}

	fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
		fields.add_field_method_get("cursor", |_, this| Ok(this.cursor()));
		fields.add_field_method_get("filetree", |_, this| Ok(this.filetree()));
		// fields.add_field_method_get("users", |_, this| Ok(this.0.users())); // TODO
	}
}

impl LuaUserData for CodempCursorController {
	fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(format!("{:?}", this)));
		methods.add_method("send", |_, this, (buffer, start_row, start_col, end_row, end_col):(String, i32, i32, i32, i32)| {
			Ok(this.send(CodempCursor { buffer, start: (start_row, start_col), end: (end_row, end_col), user: None })?)
		});
		methods.add_method("try_recv", |_, this, ()| {
			match this.try_recv()? {
				Some(x) => Ok(Some(x)),
				None => Ok(None),
			}
		});
		methods.add_method("poll", |_, this, ()| {
			STATE.rt().block_on(this.poll())?;
			Ok(())
		});
	}
}

impl LuaUserData for Cursor {
	fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
		fields.add_field_method_get("user", |_, this| Ok(this.user.map(|x| x.to_string())));
		fields.add_field_method_get("buffer", |_, this| Ok(this.buffer.clone()));
		fields.add_field_method_get("start",  |_, this| Ok(RowCol::from(this.start)));
		fields.add_field_method_get("finish", |_, this| Ok(RowCol::from(this.end)));
	}
}

pub struct RowCol {
	pub row: i32,
	pub col: i32,
}

impl From<(i32, i32)> for RowCol {
	fn from((row, col): (i32, i32)) -> Self {
		Self { row, col }
	}
}

impl LuaUserData for RowCol {
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
				this.send(
					CodempTextChange {
						span: start..end,
						content: text,
					}
				)?
			)
		});
		methods.add_method("send_diff", |_, this, (content,):(String,)| {
			Ok(
				this.send(
					CodempTextChange::from_diff(&this.content(), &content)
				)?
			)
		});
		methods.add_method("try_recv", |_, this, ()| {
			match this.try_recv()? {
				Some(x) => Ok(Some(x)),
				None => Ok(None),
			}
		});
		methods.add_method("poll", |_, this, ()| {
			STATE.rt().block_on(this.poll())?;
			Ok(())
		});
	}

	fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
		fields.add_field_method_get("content", |_, this| Ok(this.content()));
	}
}

impl LuaUserData for CodempOp { }

impl LuaUserData for CodempTextChange {
	fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
		fields.add_field_method_get("content", |_, this| Ok(this.content.clone()));
		fields.add_field_method_get("first",   |_, this| Ok(this.span.start));
		fields.add_field_method_get("last",  |_, this| Ok(this.span.end));
	}

	fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_meta_function(LuaMetaMethod::Call, |_, (start, end, txt): (usize, usize, String)| {
			Ok(CodempTextChange {
				span: start..end,
				content: txt,
			})
		});
		methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(format!("{:?}", this)));
		methods.add_method("apply", |_, this, (txt,):(String,)| Ok(this.apply(&txt)));
	}
}



// setup library logging to file
#[derive(Debug, derive_more::From)]
struct LuaLogger(broadcast::Receiver<String>);
impl LuaUserData for LuaLogger {
	fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_method_mut("recv", |_, this, ()| {
			Ok(this.0.blocking_recv().expect("logger channel closed"))
		});
	}
}

#[derive(Debug, Clone)]
struct LuaLoggerProducer;
impl Write for LuaLoggerProducer {
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		let _ = LOG.send(String::from_utf8_lossy(buf).to_string());
		Ok(buf.len())
	}

	fn flush(&mut self) -> std::io::Result<()> { Ok(()) }
}

fn setup_logger(_: &Lua, (debug, path): (Option<bool>, Option<String>)) -> LuaResult<()> {
	if ONCE.load(std::sync::atomic::Ordering::Relaxed) { return Ok(()) }

	let format = tracing_subscriber::fmt::format()
		.with_level(true)
		.with_target(true)
		.with_thread_ids(false)
		.with_thread_names(false)
		.with_ansi(false)
		.with_file(false)
		.with_line_number(false)
		.with_source_location(false)
		.compact();

	let level = if debug.unwrap_or_default() { tracing::Level::DEBUG } else {tracing::Level::INFO };

	let builder = tracing_subscriber::fmt()
		.event_format(format)
		.with_max_level(level);

	if let Some(path) = path {
		let logfile = std::fs::File::create(path).expect("failed creating logfile");
		builder.with_writer(Mutex::new(logfile)).init();
	} else {
		builder.with_writer(Mutex::new(LuaLoggerProducer)).init();
	}

	ONCE.store(true, std::sync::atomic::Ordering::Relaxed);
	Ok(())
}

fn get_logger(_: &Lua, (): ()) -> LuaResult<LuaLogger> {
	let sub = LOG.subscribe();
	Ok(LuaLogger(sub))
}

// define module and exports
#[mlua::lua_module]
fn codemp_lua(lua: &Lua) -> LuaResult<LuaTable> {
	let exports = lua.create_table()?;

	// core proto functions
	exports.set("login", lua.create_function(login)?)?;
	exports.set("join_workspace", lua.create_function(join_workspace)?)?;
	// state helpers
	exports.set("get_workspace", lua.create_function(get_workspace)?)?;
	// debug
	exports.set("id", lua.create_function(id)?)?;
	exports.set("get_logger", lua.create_function(get_logger)?)?;
	exports.set("setup_logger", lua.create_function(setup_logger)?)?;

	Ok(exports)
}

