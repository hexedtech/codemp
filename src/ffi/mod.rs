//! # FFI
//! The glue code for FFI (Foreign Function Interface) in various languages, each gated behind
//! a feature flag.
//!
//! For all except Java, the resulting shared object is ready to use, but external packages are
//! available to simplify dependency management and provide type hints in editor.
//!
//! ## Lua
//! Using [mlua](https://docs.rs/mlua) it's possible to map almost perfectly the entirety of `codemp` API.
//! Notable outliers are functions that receive `codemp` objects: these instead receive arguments
//! to build the object instead (such as [`crate::api::Controller::send`])
//!
//! Note that async operations are carried out on a [tokio] current_thread runtime, so it is
//! necessary to drive it. A separate driver thread can be spawned with `spawn_runtime_driver`
//! function.
//!
//! To work with callbacks, the main Lua thread must periodically stop and poll for callbacks via
//! `poll_callback`, otherwise those will never run. This is necessary to allow safe concurrent
//! access to the global Lua state, so minimize callback execution time as much as possible.
//!
//! ## Python
//! Using [pyo3](https://docs.rs/pyo3) it's possible to map perfectly the entirety of `codemp` API.
//! Async operations run on a dedicated [tokio] runtime
//!
//! ## JavaScript
//! Using [napi](https://docs.rs/napi) it's possible to map perfectly the entirety of `codemp` API.
//! Async operations run on a dedicated [tokio] runtime and the result is sent back to main thread
//!
//! ## Java
//! Since for java it is necessary to deal with the JNI and no complete FFI library is available,
//! java glue directly writes JNI functions leveraging [jni](https://docs.rs/jni) rust bindings.
//!
//! To have a runnable `jar`, some extra Java code must be compiled (available under `dist/java`)
//! and bundled together with the shared object. Such extra wrapper provides classes and methods
//! loading the native extension and invoking the underlying native functions.

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
#[cfg(feature = "python")]
pub mod python;
