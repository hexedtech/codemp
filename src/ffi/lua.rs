use std::io::Write;
use std::sync::{mpsc, Arc, Mutex};

use crate::prelude::*;
use codemp_proto::{files::BufferNode, cursor::{RowCol, CursorPosition, CursorEvent}};
use woot::crdt::Op;
use mlua::prelude::*;
use tokio::runtime::Runtime;

lazy_static::lazy_static!{
	// TODO use a runtime::Builder::new_current_thread() runtime to not behave like malware
	static ref STATE : GlobalState = GlobalState::default();
}

struct GlobalState {
	client: std::sync::RwLock<CodempClient>,
	runtime: Runtime,
}

impl Default for GlobalState {
	fn default() -> Self {
		let rt = Runtime::new().expect("could not create tokio runtime");
		let client = rt.block_on(
			CodempClient::new("http://codemp.alemi.dev:50053")
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

#[derive(Debug, thiserror::Error, derive_more::From, derive_more::Display)]
struct LuaCodempError(CodempError);

impl From::<LuaCodempError> for LuaError {
	fn from(value: LuaCodempError) -> Self {
		LuaError::external(value)
	}
}

// TODO put friendlier constructor directly in lib?
fn make_cursor(buffer: String, start_row: i32, start_col: i32, end_row: i32, end_col: i32) -> CursorPosition {
	CursorPosition {
		buffer: BufferNode::from(buffer), start: RowCol { row: start_row, col: start_col}, end: RowCol { row: end_row, col: end_col },
	}
}

fn id(_: &Lua, (): ()) -> LuaResult<String> {
	Ok(STATE.client().user_id().to_string())
}


/// join a remote workspace and start processing cursor events
fn join_workspace(_: &Lua, (session,): (String,)) -> LuaResult<LuaCursorController> {
	tracing::info!("joining workspace {}", session);
	let ws = STATE.rt().block_on(async { STATE.client_mut().join_workspace(&session).await })
		.map_err(LuaCodempError::from)?;
	let cursor = ws.cursor();
	Ok(cursor.into())
}

fn login(_: &Lua, (username, password, workspace_id):(String, String, String)) -> LuaResult<()> {
	Ok(STATE.rt().block_on(STATE.client().login(username, password, Some(workspace_id))).map_err(LuaCodempError::from)?)
}

fn get_workspace(_: &Lua, (session,): (String,)) -> LuaResult<Option<LuaWorkspace>> {
	Ok(STATE.client().get_workspace(&session).map(LuaWorkspace))
}

#[derive(Debug, derive_more::From)]
struct LuaOp(Op);
impl LuaUserData for LuaOp { }

#[derive(derive_more::From)]
struct LuaWorkspace(Arc<CodempWorkspace>);
impl LuaUserData for LuaWorkspace {
	fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_method("create_buffer", |_, this, (name,):(String,)| {
			Ok(STATE.rt().block_on(async { this.0.create(&name).await }).map_err(LuaCodempError::from)?)
		});

		methods.add_method("attach_buffer", |_, this, (name,):(String,)| {
			Ok(LuaBufferController(STATE.rt().block_on(async { this.0.attach(&name).await }).map_err(LuaCodempError::from)?))
		});

		// TODO disconnect_buffer
		// TODO leave_workspace:w

		methods.add_method("get_buffer", |_, this, (name,):(String,)| Ok(this.0.buffer_by_name(&name).map(LuaBufferController)));
	}

	fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
		fields.add_field_method_get("cursor", |_, this| Ok(LuaCursorController(this.0.cursor())));
		fields.add_field_method_get("filetree", |_, this| Ok(this.0.filetree()));
		// fields.add_field_method_get("users", |_, this| Ok(this.0.users())); // TODO
	}
}



#[derive(Debug, derive_more::From)]
struct LuaCursorController(Arc<CodempCursorController>);
impl LuaUserData for LuaCursorController {
	fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(format!("{:?}", this.0)));
		methods.add_method("send", |_, this, (usr, sr, sc, er, ec):(String, i32, i32, i32, i32)| {
			Ok(this.0.send(make_cursor(usr, sr, sc, er, ec)).map_err(LuaCodempError::from)?)
		});
		methods.add_method("try_recv", |_, this, ()| {
			match this.0.try_recv() .map_err(LuaCodempError::from)? {
				Some(x) => Ok(Some(LuaCursorEvent(x))),
				None => Ok(None),
			}
		});
		methods.add_method("poll", |_, this, ()| {
			STATE.rt().block_on(this.0.poll())
					.map_err(LuaCodempError::from)?;
			Ok(())
		});
	}
}

#[derive(Debug, derive_more::From)]
struct LuaCursorEvent(CursorEvent);
impl LuaUserData for LuaCursorEvent {
	fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
		fields.add_field_method_get("user", |_, this| Ok(this.0.user.id.clone()));
		fields.add_field_method_get("position", |_, this|
			Ok(LuaCursorPosition(this.0.position.clone()))
		);
	}
}

#[derive(Debug, derive_more::From)]
struct LuaCursorPosition(CursorPosition);
impl LuaUserData for LuaCursorPosition {
	fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
		fields.add_field_method_get("buffer", |_, this| Ok(this.0.buffer.path.clone()));
		fields.add_field_method_get("start",  |_, this| Ok(LuaRowCol(this.0.start.clone())));
		fields.add_field_method_get("finish", |_, this| Ok(LuaRowCol(this.0.end.clone())));
	}
}


#[derive(Debug, derive_more::From)]
struct LuaBufferController(Arc<CodempBufferController>);
impl LuaUserData for LuaBufferController {
	fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(format!("{:?}", this.0)));
		methods.add_method("send", |_, this, (start, end, text): (usize, usize, String)| {
			Ok(
				this.0.send(
					CodempTextChange {
						span: start..end,
						content: text,
					}
				)
					.map_err(LuaCodempError::from)?
			)
		});
		methods.add_method("send_diff", |_, this, (content,):(String,)| {
			Ok(
				this.0.send(
					CodempTextChange::from_diff(&this.0.content(), &content)
				)
					.map_err(LuaCodempError::from)?
			)
		});
		methods.add_method("try_recv", |_, this, ()| {
			match this.0.try_recv().map_err(LuaCodempError::from)? {
				Some(x) => Ok(Some(LuaTextChange(x))),
				None => Ok(None),
			}
		});
		methods.add_method("poll", |_, this, ()| {
			STATE.rt().block_on(this.0.poll())
					.map_err(LuaCodempError::from)?;
			Ok(())
		});
	}

	fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
		fields.add_field_method_get("content", |_, this| Ok(this.0.content()));
	}
}

#[derive(Debug, derive_more::From)]
struct LuaTextChange(CodempTextChange);
impl LuaUserData for LuaTextChange {
	fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
		fields.add_field_method_get("content", |_, this| Ok(this.0.content.clone()));
		fields.add_field_method_get("first",   |_, this| Ok(this.0.span.start));
		fields.add_field_method_get("last",  |_, this| Ok(this.0.span.end));
	}

	fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_meta_function(LuaMetaMethod::Call, |_, (start, end, txt): (usize, usize, String)| {
			Ok(LuaTextChange(CodempTextChange {
				span: start..end,
				content: txt,
			}))
		});
		methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(format!("{:?}", this.0)));
		methods.add_method("apply", |_, this, (txt,):(String,)| Ok(this.0.apply(&txt)));
	}
}

#[derive(Debug, derive_more::From)]
struct LuaRowCol(RowCol);
impl LuaUserData for LuaRowCol {
	fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
		fields.add_field_method_get("row", |_, this| Ok(this.0.row));
		fields.add_field_method_get("col", |_, this| Ok(this.0.col));
	}
}



// setup library logging to file
#[derive(Debug, derive_more::From)]
struct LuaLogger(Arc<Mutex<mpsc::Receiver<String>>>);
impl LuaUserData for LuaLogger {
	fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_method("recv", |_, this, ()| {
			Ok(
				this.0
					.lock()
					.expect("logger mutex poisoned")
					.recv()
					.expect("logger channel closed")
			)
		});
	}
}

#[derive(Debug, Clone)]
struct LuaLoggerProducer(mpsc::Sender<String>);
impl Write for LuaLoggerProducer {
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		self.0.send(String::from_utf8_lossy(buf).to_string())
			.expect("could not write on logger channel");
		Ok(buf.len())
	}

	fn flush(&mut self) -> std::io::Result<()> { Ok(()) }
}

fn setup_tracing(_: &Lua, (debug,): (Option<bool>,)) -> LuaResult<LuaLogger> {
	let (tx, rx) = mpsc::channel();
	let level = if debug.unwrap_or(false) { tracing::Level::DEBUG } else {tracing::Level::INFO };
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
	tracing_subscriber::fmt()
		.event_format(format)
		.with_max_level(level)
		.with_writer(Mutex::new(LuaLoggerProducer(tx)))
		.init();
	Ok(LuaLogger(Arc::new(Mutex::new(rx))))
}

// define module and exports
#[mlua::lua_module]
fn libcodemp(lua: &Lua) -> LuaResult<LuaTable> {
	let exports = lua.create_table()?;

	// core proto functions
	exports.set("login", lua.create_function(login)?)?;
	exports.set("join_workspace", lua.create_function(join_workspace)?)?;
	// state helpers
	exports.set("get_workspace", lua.create_function(get_workspace)?)?;
	// debug
	exports.set("id", lua.create_function(id)?)?;
	exports.set("setup_tracing", lua.create_function(setup_tracing)?)?;

	Ok(exports)
}


