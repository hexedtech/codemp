//! ### FFI
//! The glue code for FFI (Foreign Function Interface) in various languages, each gated behind
//! a feature flag.
//!
//! For all except Java, the resulting shared object is ready to use, but external packages are
//! available to simplify dependency management and provide type hints in editor.

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
