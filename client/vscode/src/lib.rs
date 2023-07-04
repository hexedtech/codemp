use std::sync::Arc;

use neon::prelude::*;
use once_cell::sync::OnceCell;
use codemp::{
	cursor::{CursorControllerHandle, CursorSubscriber}, client::CodempClient, operation::{OperationController, OperationFactory, OperationProcessor},
	proto::buffer_client::BufferClient,
};
use codemp::tokio::{runtime::Runtime, sync::Mutex};

fn runtime<'a, C: Context<'a>>(cx: &mut C) -> NeonResult<&'static Runtime> {
	static RUNTIME: OnceCell<Runtime> = OnceCell::new();

	RUNTIME.get_or_try_init(|| {
		Runtime::new()
			.or_else(|err| cx.throw_error(err.to_string()))
	})
}

fn tuple<'a, C: Context<'a>>(cx: &mut C, a: i32, b: i32) -> NeonResult<Handle<'a, JsArray>> {
	let obj = cx.empty_array();
	let a_val = cx.number(a);
	obj.set(cx, 0, a_val)?;
	let b_val = cx.number(b);
	obj.set(cx, 1, b_val)?;
	Ok(obj)
}

fn unpack_tuple<'a, C: Context<'a>>(cx: &mut C, arr: Handle<'a, JsArray>) -> NeonResult<(i32, i32)> {
	Ok((
		arr.get::<JsNumber, _, u32>(cx, 0)?.value(cx) as i32,
		arr.get::<JsNumber, _, u32>(cx, 1)?.value(cx) as i32,
	))
}

struct ClientHandle(Arc<Mutex<CodempClient>>);
impl Finalize for ClientHandle {}

fn connect(mut cx: FunctionContext) -> JsResult<JsPromise> {
	let host = cx.argument::<JsString>(0).ok().map(|x| x.value(&mut cx));

	let (deferred, promise) = cx.promise();
	let channel = cx.channel();

	runtime(&mut cx)?.spawn(async move {
		match BufferClient::connect(host.unwrap_or("".into())).await {
			Err(e) => deferred.settle_with(&channel, move |mut cx| cx.throw_error::<String, neon::handle::Handle<JsString>>(format!("{}", e))),
			Ok(c) => deferred.settle_with(&channel, |mut cx| {
				let obj = cx.empty_object();
				let boxed_value = cx.boxed(ClientHandle(Arc::new(Mutex::new(c.into()))));
				obj.set(&mut cx, "boxed", boxed_value)?;
				let method_create = JsFunction::new(&mut cx, create_client)?;
				obj.set(&mut cx, "create", method_create)?;
				let method_listen = JsFunction::new(&mut cx, listen_client)?;
				obj.set(&mut cx, "listen", method_listen)?;
				let method_attach = JsFunction::new(&mut cx, attach_client)?;
				obj.set(&mut cx, "attach", method_attach)?;
				Ok(obj)
			}),
		}
	});

	Ok(promise)
}

fn create_client(mut cx: FunctionContext) -> JsResult<JsPromise> {
	let path = cx.argument::<JsString>(0)?.value(&mut cx);
	let content = cx.argument::<JsString>(1).ok().map(|x| x.value(&mut cx));
	let this = cx.this();
	let boxed : Handle<JsBox<ClientHandle>> = this.get(&mut cx, "boxed")?;

	let rc = boxed.0.clone();
	let (deferred, promise) = cx.promise();
	let channel = cx.channel();

	runtime(&mut cx)?.spawn(async move {
		match rc.lock().await.create(path, content).await {
			Ok(accepted) => deferred.settle_with(&channel, move |mut cx| Ok(cx.boolean(accepted))),
			Err(e) => deferred.settle_with(&channel, move |mut cx| cx.throw_error::<String, neon::handle::Handle<JsString>>(e.to_string())),
		}
	});

	Ok(promise)
}

fn listen_client(mut cx: FunctionContext) -> JsResult<JsPromise> {
	let this = cx.this();
	let boxed : Handle<JsBox<ClientHandle>> = this.get(&mut cx, "boxed")?;

	let rc = boxed.0.clone();
	let (deferred, promise) = cx.promise();
	let channel = cx.channel();

	runtime(&mut cx)?.spawn(async move {
		match rc.lock().await.listen().await {
			Ok(controller) => {
				deferred.settle_with(&channel, move |mut cx| {
					let obj = cx.empty_object();
					let boxed_value = cx.boxed(CursorEventsHandle(controller));
					obj.set(&mut cx, "boxed", boxed_value)?;
					let callback_method = JsFunction::new(&mut cx, callback_cursor)?;
					obj.set(&mut cx, "callback", callback_method)?;
					let send_method = JsFunction::new(&mut cx, send_cursor)?;
					obj.set(&mut cx, "send", send_method)?;
					Ok(obj)
				})
			},
			Err(e) => deferred.settle_with(&channel, move |mut cx| cx.throw_error::<String, neon::handle::Handle<JsString>>(e.to_string())),
		}
	});

	Ok(promise)
}

fn attach_client(mut cx: FunctionContext) -> JsResult<JsPromise> {
	let this = cx.this();
	let boxed : Handle<JsBox<ClientHandle>> = this.get(&mut cx, "boxed")?;
	let path = cx.argument::<JsString>(0)?.value(&mut cx);

	let rc = boxed.0.clone();
	let (deferred, promise) = cx.promise();
	let channel = cx.channel();

	runtime(&mut cx)?.spawn(async move {
		match rc.lock().await.attach(path).await {
			Ok(controller) => {
				deferred.settle_with(&channel, move |mut cx| {
					let obj = cx.empty_object();
					let boxed_value = cx.boxed(OperationControllerHandle(controller));
					obj.set(&mut cx, "boxed", boxed_value)?;
					let apply_method = JsFunction::new(&mut cx, apply_operation)?;
					obj.set(&mut cx, "apply", apply_method)?;
					let content_method = JsFunction::new(&mut cx, content_operation)?;
					obj.set(&mut cx, "content", content_method)?;
					let callback_method = JsFunction::new(&mut cx, callback_operation)?;
					obj.set(&mut cx, "callback", callback_method)?;
					Ok(obj)
				})
			},
			Err(e) => deferred.settle_with(&channel, move |mut cx| cx.throw_error::<String, neon::handle::Handle<JsString>>(e.to_string())),
		}
	});

	Ok(promise)
}


struct OperationControllerHandle(Arc<OperationController>);
impl Finalize for OperationControllerHandle {}

fn apply_operation(mut cx: FunctionContext) -> JsResult<JsPromise> {
	let this = cx.this();
	let boxed : Handle<JsBox<OperationControllerHandle>> = this.get(&mut cx, "boxed")?;
	let skip = cx.argument::<JsNumber>(0)?.value(&mut cx).round() as usize;
	let text = cx.argument::<JsString>(1)?.value(&mut cx);
	let tail = cx.argument::<JsNumber>(2)?.value(&mut cx).round() as usize;

	let rc = boxed.0.clone();
	let (deferred, promise) = cx.promise();
	let channel = cx.channel();

	runtime(&mut cx)?.spawn(async move {
		let op = rc.delta(skip, text.as_str(), tail);
		match rc.apply(op) {
			Err(e) => deferred.settle_with(&channel, move |mut cx| cx.throw_error::<_, Handle<JsString>>(format!("could not apply operation: {}", e))),
			Ok(span) => deferred.settle_with(&channel, move |mut cx| tuple(&mut cx, span.start as i32, span.end as i32)),
		}
	});

	Ok(promise)
}

fn content_operation(mut cx: FunctionContext) -> JsResult<JsString> {
	let this = cx.this();
	let boxed : Handle<JsBox<OperationControllerHandle>> = this.get(&mut cx, "boxed")?;
	Ok(cx.string(boxed.0.content()))
}

fn callback_operation(mut cx: FunctionContext) -> JsResult<JsUndefined> {
	let this = cx.this();
	let boxed : Handle<JsBox<OperationControllerHandle>> = this.get(&mut cx, "boxed")?;
	let callback = Arc::new(cx.argument::<JsFunction>(0)?.root(&mut cx));

	let rc = boxed.0.clone();
	let channel = cx.channel();

	// TODO when garbage collecting OperationController stop this worker
	runtime(&mut cx)?.spawn(async move {
		loop{
			let span = rc.wait().await;
			let cb = callback.clone();
			channel.send(move |mut cx| {
				cb.to_inner(&mut cx)
					.call_with(&cx)
					.arg(cx.number(span.start as i32))
					.arg(cx.number(span.end as i32))
					.apply::<JsUndefined, _>(&mut cx)?;
				Ok(())
			});
		}
	});

	Ok(cx.undefined())
}

struct CursorEventsHandle(CursorControllerHandle);
impl Finalize for CursorEventsHandle {}

fn callback_cursor(mut cx: FunctionContext) -> JsResult<JsUndefined> {
	let this = cx.this();
	let boxed : Handle<JsBox<CursorEventsHandle>> = this.get(&mut cx, "boxed")?;
	let callback = Arc::new(cx.argument::<JsFunction>(0)?.root(&mut cx));

	let mut rc = boxed.0.clone();
	let channel = cx.channel();

	// TODO when garbage collecting OperationController stop this worker
	runtime(&mut cx)?.spawn(async move {
		while let Some(op) = rc.poll().await {
			let cb = callback.clone();
			channel.send(move |mut cx| {
				cb.to_inner(&mut cx)
					.call_with(&cx)
					.arg(cx.string(op.user))
					.arg(cx.string(op.buffer))
					.arg(tuple(&mut cx, op.start.row as i32, op.start.col as i32)?)
					.arg(tuple(&mut cx, op.end.row as i32, op.end.col as i32)?)
					.apply::<JsUndefined, _>(&mut cx)?;
				Ok(())
			});
		}

	});

	Ok(cx.undefined())
}

fn send_cursor(mut cx: FunctionContext) -> JsResult<JsPromise> {
	let this = cx.this();
	let boxed : Handle<JsBox<CursorEventsHandle>> = this.get(&mut cx, "boxed")?;
	let path = cx.argument::<JsString>(0)?.value(&mut cx);
	let start_obj = cx.argument::<JsArray>(1)?;
	let start = unpack_tuple(&mut cx, start_obj)?;
	let end_obj = cx.argument::<JsArray>(2)?;
	let end = unpack_tuple(&mut cx, end_obj)?;

	let rc = boxed.0.clone();
	let (deferred, promise) = cx.promise();
	let channel = cx.channel();

	runtime(&mut cx)?.spawn(async move {
		rc.send(&path, start.into(), end.into()).await;
		deferred.settle_with(&channel, |mut cx| Ok(cx.undefined()))
	});

	Ok(promise)
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
	cx.export_function("connect", connect)?;

	Ok(())
}
