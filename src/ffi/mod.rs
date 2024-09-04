//! ### FFI
//! Foreign-Function-Interface glue code, each gated behind feature flags
//!
//! For all except java, the resulting shared object is ready to use, but external packages are
//! available to simplify the dependancy and provide type hints in editor.

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
