use mlua_codemp_patch as mlua;
use mlua::prelude::*;

pub(crate) fn tokio() -> &'static tokio::runtime::Runtime {
	use std::sync::OnceLock;
	static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
	RT.get_or_init(||
		tokio::runtime::Builder::new_current_thread()
			.enable_all()
			.build()
			.expect("could not create tokio runtime")
	)
}

macro_rules! a_sync {
	($($clone:ident)* => $x:expr) => {
		{
			$(let $clone = $clone.clone();)*
			Ok(
				crate::ffi::lua::ext::a_sync::Promise(
					Some(
						crate::ffi::lua::ext::a_sync::tokio()
							.spawn(async move {
								Ok(crate::ffi::lua::ext::callback::CallbackArg::from($x))
							})
					)
				)
			)
		}
	};
}

pub(crate) use a_sync;

pub(crate) struct Promise(pub(crate) Option<tokio::task::JoinHandle<LuaResult<super::callback::CallbackArg>>>);

impl LuaUserData for Promise {
	fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("ready", |_, this|
			Ok(this.0.as_ref().map_or(true, |x| x.is_finished()))
		);
	}

	fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
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
							Ok(res) => match res {
								Err(e) => super::callback().failure(e),
								Ok(val) => super::callback().invoke(cb, val),
							},
						}
					});
				Ok(())
			},
		});
	}
}

pub(crate) fn setup_driver(_: &Lua, (block,):(Option<bool>,)) -> LuaResult<Option<Driver>> {
	let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
	let future = async move {
		tracing::info!(" :: driving runtime...");
		tokio::select! {
			() = std::future::pending::<()>() => {},
			_ = rx.recv() => {},
		}
	};
	if block.unwrap_or(false) {
		super::tokio().block_on(future);
		Ok(None)
	} else {
		let handle = std::thread::spawn(move || super::tokio().block_on(future));
		Ok(Some(Driver(tx, Some(handle))))
	}
}

#[derive(Debug)]
pub(crate) struct Driver(pub(crate) tokio::sync::mpsc::UnboundedSender<()>, Option<std::thread::JoinHandle<()>>);
impl LuaUserData for Driver {
	fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(format!("{:?}", this)));
		methods.add_method_mut("stop", |_, this, ()| {
			match this.1.take() {
				None => Ok(false),
				Some(handle) => {
					if this.0.send(()).is_err() {
						tracing::warn!("found runtime already stopped while attempting to stop it");
					}
					match handle.join() {
						Err(e) => Err(LuaError::runtime(format!("runtime thread panicked: {e:?}"))),
						Ok(()) => Ok(true),
					}
				},
			}
		});
	}
}


