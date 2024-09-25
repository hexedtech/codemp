#[cfg(feature = "js")]
extern crate napi_build;

#[cfg(any(feature = "py", feature = "py-noabi"))]
extern crate pyo3_build_config;

/// The main method of the buildscript, required by some glue modules.
fn main() {
	#[cfg(feature = "js")]
	{
		napi_build::setup();
	}

	#[cfg(any(feature = "py", feature = "py-noabi"))]
	{
		pyo3_build_config::add_extension_module_link_args();
	}

	#[cfg(feature = "lua")]
	{
		if let Ok("macos") = std::env::var("CARGO_CFG_TARGET_OS").as_deref() {
			println!("cargo:rustc-cdylib-link-arg=-undefined");
  		println!("cargo:rustc-cdylib-link-arg=dynamic_lookup");
  	}
	}
}
