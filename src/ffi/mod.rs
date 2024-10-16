//! # Foreign Function Interface
//! `codemp` aims to be available as a library from as many programming languages as possible.
//! To achieve this, we rely on Foreign Function Interface.
//!
//! ```no_run
//! # async {
//! use codemp::api::controller::{AsyncReceiver, AsyncSender}; // needed for send/recv trait methods
//!
//! // connect first, api.code.mp is managed by hexed.technology
//! let client = codemp::Client::connect(codemp::api::Config {
//!   username: "mail@example.net".into(), password: "dont-use-this-password".into(),
//!   ..Default::default()
//! }).await?;
//!
//! // create and join a workspace
//! client.create_workspace("some-workspace").await?;
//! let workspace = client.attach_workspace("some-workspace").await?;
//!
//! // create a new buffer in this workspace and attach to it
//! workspace.create_buffer("/my/file.txt").await?;
//! let buffer = workspace.attach_buffer("/my/file.txt").await?;
//!
//! // write `hello!` at the beginning of this buffer
//! buffer.send(codemp::api::TextChange {
//!   start_idx: 0, end_idx: 0,
//!   content: "hello!".to_string(),
//! })?;
//!
//! // wait for cursor movements
//! loop {
//!   let event = workspace.cursor().recv().await?;
//!   println!("user {} moved on buffer {}", event.user, event.sel.buffer);
//! }
//! # Ok::<(),Box<dyn std::error::Error>>(())
//! # };
//! ```
//!
//! ## JavaScript
//! Our JavaScript glue is built with [`napi`](https://napi.rs).
//!
//! All async operations are handled on a separate tokio runtime, automatically managed by `napi`.
//! Callbacks are safely scheduled to be called on the main loop thread.
//!
//! ```js
//! import * as codemp from 'codemp';
//!
//! // connect first, api.code.mp is managed by hexed.technology
//! let client = await codemp.connect({
//!   username: "mail@example.net",
//!   password: "dont-use-this-password"
//! });
//!
//! // create and join a workspace
//! await client.createWorkspace("some-workspace");
//! let workspace = await client.attachWorkspace("some-workspace");
//!
//! // create a new buffer in this workspace and attach to it
//! await workspace.createBuffer("/my/file.txt");
//! let buffer = await workspace.attachBuffer("/my/file.txt");
//!
//! // write `hello!` at the beginning of this buffer
//! await buffer.send({
//!   start_idx: 0, end_idx: 0,
//!   content: "hello!",
//! });
//!
//! // wait for cursor movements
//! while (true) {
//!   let event = await workspace.cursor().recv();
//!   console.log(`user ${event.user} moved on buffer ${event.buffer}`);
//! }
//! ```
//!
//! ## Python
//! Our Python glue is built with [`PyO3`](https://pyo3.rs).
//!
//! All async operations return a `Promise`, which can we `.wait()`-ed to block and get the return
//! value. The `Future` itself is run on a `tokio` runtime in a dedicated thread, which must be
//! stared with `codemp.init()` before doing any async operations.
//!
//! ```py
//! import codemp
//!
//! # connect first, api.code.mp is managed by hexed.technology
//! client = codemp.connect(codemp.Config(
//!   username = "mail@example.net",
//!   password = "dont-use-this-password"
//! )).wait()
//!
//! # create and join a workspace
//! client.create_workspace("some-workspace").wait()
//! workspace = client.attach_workspace("some-workspace").wait()
//!
//! # create a new buffer in this workspace and attach to it
//! workspace.create_buffer("/my/file.txt").wait()
//! buffer = workspace.attach_buffer("/my/file.txt").wait()
//!
//! # write `hello!` at the beginning of this buffer
//! buffer.send(codemp.TextChange(
//!   start_idx=0, end_idx=0,
//!   content="hello!"
//! )).wait()
//!
//! # wait for cursor movements
//! while true:
//!   event = workspace.cursor().recv().wait()
//!   print(f"user {event.user} moved on buffer {event.buffer}")
//!
//! ```
//!
//! ## Lua
//! Our Lua glue is built with [`mlua`](https://github.com/mlua-rs/mlua).
//!
//! Lua bindings run all async code on a current thread tokio runtime, which should be driven with
//! a dedicated thread.
//!
//! All async functions will return a `Promise`, which can be `:await()`-ed to block and get the
//! return value.
//!
//! Note as Lua uses filename to locate entrypoint symbol, so shared object can't just have any name.
//! Accepted filenames are `libcodemp.___`, `codemp.___`, `codemp_native.___`, `codemp_lua.___` (extension depends on your platform: `so` on linux, `dll` on windows, `dylib` on macos).
//! Type hints are provided in `dist/lua/annotations.lua`, just include them in your language server: `---@module 'annotations'`.
//!
//! `codemp` is available as a rock on [LuaRocks](https://luarocks.org/modules/alemi/codemp),
//! however LuaRocks compiles from source and will require having `cargo` installed.
//! We provide pre-built binaries at [codemp.dev/releases/lua](https://codemp.dev/releases/lua/).
//! **Please do not rely on this link, as our built binaries will likely move somewhere else soon!**.
//!
//! ```lua
//! CODEMP = require('codemp')
//!
//! -- connect first, api.code.mp is managed by hexed.technology
//! local client = CODEMP.connect({
//!   username = "mail@example.net",
//!   password = "dont-use-this-password"
//! }):await()
//!
//! -- create and join a workspace
//! client:create_workspace("my-workspace"):await()
//! local workspace = client:attach_workspace("my-workspace"):await()
//!
//! -- create a new buffer in this workspace and attach to it
//! workspace:create_buffer("/my/file.txt"):await()
//! local buffer = workspace:attach_buffer("/my/file.txt"):await()
//!
//! -- write `hello!` at the beginning of this buffer
//! buffer:send({
//!   start_idx = 0, end_idx = 0,
//!   content = "hello!"
//! }):await()
//!
//! -- wait for cursor movements
//! while true do
//!   local event = workspace.cursor:recv():await()
//!   print("user " .. event.user .. " moved on buffer " .. event.buffer)
//! end
//! ```
//!
//! ## Java
//! Our Java glue is built with [`jni`](https://github.com/jni-rs/jni-rs).
//!
//! Memory management is entirely delegated to the JVM's garbage collector.
//! A more elegant solution than `Object.finalize()`, who is deprecated in newer Java versions, may be coming eventually.
//!
//! Exceptions coming from the native side have generally been made checked to imitate Rust's philosophy with `Result`.
//! `JNIException`s are however unchecked: there is nothing you can do to recover from them, as they usually represent a severe error in the glue code. If they arise, it's probably a bug.
//!
//! ```java
//! import mp.code.*;
//!
//! // connect first, api.code.mp is managed by hexed.technology
//! Client client = Client.connect(new data.Config(
//!   "mail@example.net",
//!   "dont-use-this-password"
//! ));
//!
//! // create and join a workspace
//! client.createWorkspace("some-workspace");
//! Workspace workspace = client.attachWorkspace("some-workspace");
//!
//! // create a new buffer in this workspace and attach to it
//! workspace.createBuffer("/my/file.txt");
//! BufferController buffer = workspace.attachBuffer("/my/file.txt");
//!
//! // write `hello!` at the beginning of this buffer
//! buffer.send(new data.TextChange(
//!   0, 0, "hello!",
//!   java.util.OptionalLong.empty() // optional, used for error detection
//! ));
//!
//! // wait for cursor movements
//! while (true) {
//!   data.Cursor event = workspace.cursor().recv();
//!   System.out.printf("user %s moved on buffer %s\n", event.user, event.buffer);
//! }
//! ```

#![allow(clippy::unit_arg)]

/// java bindings, built with [jni]
#[cfg(feature = "java")]
pub mod java;

/// lua bindings, built with [mlua]
#[cfg(feature = "lua")]
pub mod lua;

/// javascript bindings, built with [napi]
#[cfg(feature = "js")]
pub mod js;

/// python bindings, built with [pyo3]
#[cfg(any(feature = "py", feature = "py-noabi"))]
pub mod python;
