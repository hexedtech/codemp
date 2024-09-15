//! # Foreign Function Interface
//! `codemp` aims to be available as a library from as many programming languages as possible.
//! To achieve this, we rely on Foreign Function Interface.
//! 
//! ## JavaScript
//! Our JavaScript glue is built with [`napi`](https://napi.rs).
//!
//! All async operations are handled on a separate tokio runtime, automatically managed by `napi`.
//! Callbacks are safely scheduled to be called on the main loop thread.
//! 
//! ## Python
//! Our Python glue is built with [`PyO3`](https://pyo3.rs).
//!
//! All async operations return a `Promise`, which can we `.wait()`-ed to block and get the return
//! value. The `Future` itself is run on a `tokio` runtime in a dedicated thread, which must be
//! stared with `codemp.init()` before doing any async operations.
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
//! ## Java
//! Our Java glue is built with [`jni`](https://github.com/jni-rs/jni-rs).
//!
//! Memory management is entirely delegated to the JVM's garbage collector.
//! A more elegant solution than `Object.finalize()`, who is deprecated in newer Java versions, may be coming eventually.
//! 
//! Exceptions coming from the native side have generally been made checked to imitate Rust's philosophy with `Result`.
//! `JNIException`s are however unchecked: there is nothing you can do to recover from them, as they usually represent a severe error in the glue code. If they arise, it's probably a bug.
//! 

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
#[cfg(feature = "py")]
pub mod python;
