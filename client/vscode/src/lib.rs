use std::sync::Arc;

use neon::prelude::*;
use once_cell::sync::OnceCell;
use codemp::{
	cursor::Cursor, client::CodempClient, operation::{OperationController, OperationFactory, OperationProcessor},
	proto::buffer_client::BufferClient,
};
use codemp::tokio::{runtime::Runtime, sync::{Mutex, broadcast}};

fn runtime<'a, C: Context<'a>>(cx: &mut C) -> NeonResult<&'static Runtime> {
	static RUNTIME: OnceCell<Runtime> = OnceCell::new();

	RUNTIME.get_or_try_init(|| {
		Runtime::new()
			.or_else(|err| cx.throw_error(err.to_string()))
	})
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
				let method_create = JsFunction::new(&mut cx, create)?;
				obj.set(&mut cx, "create", method_create)?;
				let method_listen = JsFunction::new(&mut cx, listen)?;
				obj.set(&mut cx, "listen", method_listen)?;
				let method_attach = JsFunction::new(&mut cx, attach)?;
				obj.set(&mut cx, "attach", method_attach)?;
				let method_cursor = JsFunction::new(&mut cx, cursor)?;
				obj.set(&mut cx, "cursor", method_cursor)?;
				let test = cx.null();
				obj.set(&mut cx, "test", test)?;
				Ok(obj)
			}),
		}
	});

	Ok(promise)
}

fn create(mut cx: FunctionContext) -> JsResult<JsPromise> {
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

fn listen(mut cx: FunctionContext) -> JsResult<JsPromise> {
	let this = cx.this();
	let boxed : Handle<JsBox<ClientHandle>> = this.get(&mut cx, "boxed")?;

	let rc = boxed.0.clone();
	let (deferred, promise) = cx.promise();
	let channel = cx.channel();

	runtime(&mut cx)?.spawn(async move {
		match rc.lock().await.listen().await {
			Ok(controller) => {
				let sub = controller.sub();
				deferred.settle_with(&channel, move |mut cx| {
					let obj = cx.empty_object();
					let boxed_value = cx.boxed(CursorEventsHandle(Arc::new(Mutex::new(sub))));
					obj.set(&mut cx, "boxed", boxed_value)?;
					let poll_method = JsFunction::new(&mut cx, poll_cursor)?;
					obj.set(&mut cx, "poll", poll_method)?;
					Ok(obj)
				})
			},
			Err(e) => deferred.settle_with(&channel, move |mut cx| cx.throw_error::<String, neon::handle::Handle<JsString>>(e.to_string())),
		}
	});

	Ok(promise)
}

fn attach(mut cx: FunctionContext) -> JsResult<JsPromise> {
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
					let poll_method = JsFunction::new(&mut cx, poll_operation)?;
					obj.set(&mut cx, "poll", poll_method)?;
					let content_method = JsFunction::new(&mut cx, content_operation)?;
					obj.set(&mut cx, "content", content_method)?;
					let callback_method = JsFunction::new(&mut cx, callback_operation)?;
					obj.set(&mut cx, "set_callback", callback_method)?;
					Ok(obj)
				})
			},
			Err(e) => deferred.settle_with(&channel, move |mut cx| cx.throw_error::<String, neon::handle::Handle<JsString>>(e.to_string())),
		}
	});

	Ok(promise)
}

fn cursor(mut cx: FunctionContext) -> JsResult<JsPromise> {
	let this = cx.this();
	let boxed : Handle<JsBox<ClientHandle>> = this.get(&mut cx, "boxed")?;

	let path = cx.argument::<JsString>(0)?.value(&mut cx);
	let row = cx.argument::<JsNumber>(1)?.value(&mut cx) as i64;
	let col = cx.argument::<JsNumber>(2)?.value(&mut cx) as i64;

	let rc = boxed.0.clone();
	let (deferred, promise) = cx.promise();
	let channel = cx.channel();

	runtime(&mut cx)?.spawn(async move {
		match rc.lock().await.cursor(path, row, col).await {
			Ok(accepted) => deferred.settle_with(&channel, move |mut cx| Ok(cx.boolean(accepted))),
			Err(e) => deferred.settle_with(&channel, move |mut cx| cx.throw_error::<_, Handle<JsString>>(e.to_string())),
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
			Ok(span) => deferred.settle_with(&channel, move |mut cx| {
				let obj = cx.empty_array();
				let start_value = cx.number(span.start as u32);
				obj.set(&mut cx, 0, start_value)?;
				let end_value = cx.number(span.end as u32);
				obj.set(&mut cx, 1, end_value)?;
				Ok(obj)
			}),
		}
	});

	Ok(promise)
}

fn poll_operation(mut cx: FunctionContext) -> JsResult<JsPromise> {
	let this = cx.this();
	let boxed : Handle<JsBox<OperationControllerHandle>> = this.get(&mut cx, "boxed")?;

	let rc = boxed.0.clone();
	let (deferred, promise) = cx.promise();
	let channel = cx.channel();

	runtime(&mut cx)?.spawn(async move {
		let span = rc.wait().await;
		deferred.settle_with(&channel, move |mut cx| {
			let obj = cx.empty_array();
			let start_value = cx.number(span.start as u32);
			obj.set(&mut cx, 0, start_value)?;
			let end_value = cx.number(span.end as u32);
			obj.set(&mut cx, 1, end_value)?;
			Ok(obj)
		});
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



struct CursorEventsHandle(Arc<Mutex<broadcast::Receiver<(String, Cursor)>>>);
impl Finalize for CursorEventsHandle {}

fn poll_cursor(mut cx: FunctionContext) -> JsResult<JsPromise> {
	let this = cx.this();
	let boxed : Handle<JsBox<CursorEventsHandle>> = this.get(&mut cx, "boxed")?;
	let rc = boxed.0.clone();

	let (deferred, promise) = cx.promise();
	let channel = cx.channel();

	runtime(&mut cx)?.spawn(async move {
		match rc.lock().await.recv().await {
			Ok((name, cursor)) => {
				deferred.settle_with(&channel, move |mut cx| {
					let obj = cx.empty_object();
					let name_value = cx.string(name);
					obj.set(&mut cx, "user", name_value)?;
					let buffer_value = cx.string(cursor.buffer);
					obj.set(&mut cx, "buffer", buffer_value)?;
					let start_value = cx.empty_array();
					let start_value_row = cx.number(cursor.start.row as i32);
					start_value.set(&mut cx, 0, start_value_row)?;
					let start_value_col = cx.number(cursor.start.col as i32);
					start_value.set(&mut cx, 0, start_value_col)?;
					obj.set(&mut cx, "start", start_value)?;
					let end_value = cx.empty_array();
					let end_value_row = cx.number(cursor.end.row as i32);
					end_value.set(&mut cx, 0, end_value_row)?;
					let end_value_col = cx.number(cursor.end.col as i32);
					end_value.set(&mut cx, 0, end_value_col)?;
					obj.set(&mut cx, "end", end_value)?;
					Ok(obj)
				})
			},
			Err(e) => deferred.settle_with(&channel, move |mut cx| cx.throw_error::<String, neon::handle::Handle<JsString>>(e.to_string())),
		}
	});

	Ok(promise)
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {

	cx.export_function("connect", connect)?;

	Ok(())
}
