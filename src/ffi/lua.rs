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

fn tokio() -> &'static tokio::runtime::Runtime {
	use std::sync::OnceLock;
	static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
	RT.get_or_init(||
		tokio::runtime::Builder::new_current_thread()
			.enable_all()
			.build()
			.expect("could not create tokio runtime")
	)
}

struct Promise<T: Send + Sync + IntoLuaMulti>(Option<tokio::task::JoinHandle<LuaResult<T>>>);

impl<T: Send + Sync + IntoLuaMulti + 'static> LuaUserData for Promise<T> {
	fn add_fields<'a, F: LuaUserDataFields<'a, Self>>(fields: &mut F) {
		fields.add_field_method_get("ready", |_, this|
			Ok(this.0.as_ref().map_or(true, |x| x.is_finished()))
		);
	}

	fn add_methods<'a, M: LuaUserDataMethods<'a, Self>>(methods: &mut M) {
		// TODO: await MUST NOT be used in callbacks!!
		methods.add_method_mut("await", |_, this, ()| match this.0.take() {
			None => Err(LuaError::runtime("Promise already awaited")),
			Some(x) => {
				tokio()
					.block_on(x)
					.map_err(LuaError::runtime)?
			},
		});
		methods.add_method_mut("and_then", |_, this, (cb,):(LuaFunction,)| match this.0.take() {
			None => Err(LuaError::runtime("Promise already awaited")),
			Some(x) => {
				tokio()
					.spawn(async move {
						match x.await {
							Err(e) => tracing::error!("could not join promise to run callback: {e}"),
							Ok(Err(e)) => tracing::error!("promise returned error: {e}"),
							Ok(Ok(res)) => {
								if let Err(e) = cb.call::<T,()>(res) {
									tracing::error!("error running promise callback: {e}");
								}
							},
						}
					});
				Ok(())
			},
		});
	}
}

macro_rules! a_sync {
	($($clone:ident)* => $x:expr) => {
		{
			$(let $clone = $clone.clone();)*
			Ok(Promise(Some(tokio().spawn(async move { $x }))))
		}
	};
}

fn spawn_runtime_driver(_: &Lua, ():()) -> LuaResult<Driver> {
	let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
	std::thread::spawn(move || tokio().block_on(async move {
		tracing::info!(" :: driving runtime...");
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
		fields.add_field_method_get("id", |_, this| Ok(this.user().id.to_string()));
		fields.add_field_method_get("username", |_, this| Ok(this.user().name.clone()));
		fields.add_field_method_get("active_workspaces", |_, this| Ok(this.active_workspaces()));
	}

	fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(format!("{:?}", this)));

		methods.add_method("refresh", |_, this, ()|
			a_sync! { this => Ok(this.refresh().await?) }
		);

		methods.add_method("join_workspace", |_, this, (ws,):(String,)|
			a_sync! { this => Ok(this.join_workspace(ws).await?) }
		);

		methods.add_method("create_workspace", |_, this, (ws,):(String,)|
			a_sync! { this => Ok(this.create_workspace(ws).await?) }
		);

		methods.add_method("delete_workspace", |_, this, (ws,):(String,)|
			a_sync! { this => Ok(this.delete_workspace(ws).await?) }
		);

		methods.add_method("invite_to_workspace", |_, this, (ws,user):(String,String)|
			a_sync! { this => Ok(this.invite_to_workspace(ws, user).await?) }
		);

		methods.add_method("list_workspaces", |_, this, (owned,invited):(Option<bool>,Option<bool>)|
			a_sync! { this => Ok(this.list_workspaces(owned.unwrap_or(true), invited.unwrap_or(true)).await?) }
		);

		methods.add_method("leave_workspace", |_, this, (ws,):(String,)|
			Ok(this.leave_workspace(&ws))
		);
		
		methods.add_method("get_workspace", |_, this, (ws,):(String,)| Ok(this.get_workspace(&ws)));
	}
}


impl LuaUserData for CodempWorkspace {
	fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(format!("{:?}", this)));
		methods.add_method("create_buffer", |_, this, (name,):(String,)|
			a_sync! { this => Ok(this.create(&name).await?) }
		);

		methods.add_method("attach", |_, this, (name,):(String,)|
			a_sync! { this => Ok(this.attach(&name).await?) }
		);

		methods.add_method("detach", |_, this, (name,):(String,)|
			Ok(matches!(this.detach(&name), DetachResult::Detaching | DetachResult::AlreadyDetached))
		);

		methods.add_method("delete_buffer", |_, this, (name,):(String,)|
			a_sync! { this => Ok(this.delete(&name).await?) }
		);

		methods.add_method("get_buffer", |_, this, (name,):(String,)| Ok(this.buffer_by_name(&name)));

		methods.add_method("event", |_, this, ()|
			a_sync! { this => Ok(this.event().await?) }
		);

		methods.add_method("fetch_buffers", |_, this, ()|
			a_sync! { this => Ok(this.fetch_buffers().await?) }
		);
		methods.add_method("fetch_users", |_, this, ()|
			a_sync! { this => Ok(this.fetch_users().await?) }
		);

		methods.add_method("callback", |_, this, (cb,):(LuaFunction,)| {
			let _this = this.clone();
			tokio().spawn(async move {
				while let Ok(ev) = _this.event().await {
					if let Err(e) = cb.call::<CodempEvent,()>(ev) {
						tracing::error!("error running workspace callback: {e}");
					}
				}
			});
			Ok(())
		});

		methods.add_method("filetree", |_, this, (filter,):(Option<String>,)|
			Ok(this.filetree(filter.as_deref()))
		);
	}

	fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
		fields.add_field_method_get("name", |_, this| Ok(this.id()));
		fields.add_field_method_get("cursor", |_, this| Ok(this.cursor()));
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
			CodempEvent::FileTreeUpdated(_) => Ok("filetree"),
			CodempEvent::UserJoin(_) | CodempEvent::UserLeave(_) => Ok("user"),
		});
		fields.add_field_method_get("value", |_, this| match this {
			CodempEvent::FileTreeUpdated(x) => Ok(x.clone()),
			CodempEvent::UserJoin(x) | CodempEvent::UserLeave(x) => Ok(x.clone()),
		});
	}
}

impl LuaUserData for CodempCursorController {
	fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(format!("{:?}", this)));

		methods.add_method("send", |_, this, (buffer, start_row, start_col, end_row, end_col):(String, i32, i32, i32, i32)|
			a_sync! { this => Ok(this.send(CodempCursor { buffer, start: (start_row, start_col), end: (end_row, end_col), user: None }).await?) }
		);
		methods.add_method("try_recv", |_, this, ()|
			a_sync! { this => Ok(this.try_recv().await?) }
		);
		methods.add_method("recv", |_, this, ()| a_sync! { this => Ok(this.recv().await?) });
		methods.add_method("poll", |_, this, ()| a_sync! { this => Ok(this.poll().await?) });

		methods.add_method("stop", |_, this, ()| Ok(this.stop()));

		methods.add_method("clear_callback", |_, this, ()| { this.clear_callback(); Ok(()) });
		methods.add_method("callback", |_, this, (cb,):(LuaFunction,)| {
			this.callback(ControllerCallback::from(move |controller: CodempCursorController| {
				if let Err(e) = cb.call::<(CodempCursorController,), ()>((controller.clone(),)) {
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

		methods.add_method("send", |_, this, (start, end, content, hash): (usize, usize, String, Option<i64>)|
			a_sync! { this => Ok(
				this.send(
					CodempTextChange {
						start: start as u32,
						end: end as u32,
						content,
						hash,
					}
				).await?
			)}
		);

		methods.add_method("try_recv", |_, this, ()| a_sync! { this => Ok(this.try_recv().await?) });
		methods.add_method("recv", |_, this, ()| a_sync! { this => Ok(this.recv().await?) });
		methods.add_method("poll", |_, this, ()| a_sync! { this => Ok(this.poll().await?) });

		methods.add_method("stop", |_, this, ()| Ok(this.stop()));

		methods.add_method("content", |_, this, ()| a_sync! { this => Ok(this.content().await?) });

		methods.add_method("clear_callback", |_, this, ()| { this.clear_callback(); Ok(()) });
		methods.add_method("callback", |_, this, (cb,):(LuaFunction,)| {
			this.callback(move |controller: CodempBufferController| {
				let _c = controller.clone();
				if let Err(e) = cb.call::<(CodempBufferController,), ()>((_c,)) {
					tracing::error!("error running buffer#{} callback: {e}", controller.name());
				}
			});
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
fn codemp_native(lua: &Lua) -> LuaResult<LuaTable> {
	let exports = lua.create_table()?;

	// entrypoint
	exports.set("connect", lua.create_function(|_, (host, username, password):(String,String,String)|
		a_sync! { => Ok(CodempClient::connect(host, username, password).await?) }
	)?)?;

	// utils
	exports.set("hash", lua.create_function(|_, (txt,):(String,)|
		Ok(crate::hash(txt))
	)?)?;

	// runtime
	exports.set("spawn_runtime_driver", lua.create_function(spawn_runtime_driver)?)?;

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
				tokio().spawn(async move {
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
