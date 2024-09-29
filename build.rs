#[cfg(feature = "js")]
extern crate napi_build;

#[cfg(any(feature = "py", feature = "py-noabi"))]
extern crate pyo3_build_config;

#[cfg(feature = "c")]
extern crate cbindgen;

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

	#[cfg(feature = "c")]
	{
		cbindgen::Builder::new()
			.with_crate(std::env::var("CARGO_MANIFEST_DIR").unwrap())
			.generate()
			.expect("Unable to generate bindings")
			.write_to_file("dist/c/codemp.h");
	}
}
