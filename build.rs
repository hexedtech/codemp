//! Buildscript, required by some glue modules for initialisation.
//! Will do nothing if no glue modules are enabled.

#[cfg(feature = "js")]
extern crate napi_build;

#[cfg(feature = "py")]
extern crate pyo3_build_config;

#[cfg(feature = "py")]
extern crate pyo3_introspection;

/// The main method of the buildscript, required by some glue modules.
fn main() {
	#[cfg(feature = "js")]
	{
		napi_build::setup();
	}

	#[cfg(feature = "py")]
	{
		pyo3_build_config::add_extension_module_link_args();

		// The Python introspection step requires an already-built cdylib, which
		// does not exist during the first clean build. Make it an opt-in second pass.
		println!("cargo:rerun-if-env-changed=CODEMP_PY_GENHINTS");
		if std::env::var("CODEMP_PY_GENHINTS").as_deref() != Ok("1") {
			return;
		}

		let out_dir = std::env::var("OUT_DIR").expect("unreachable");
		let dylib = if let Ok("windows") = std::env::var("CARGO_CFG_TARGET_OS").as_deref() {
			"libcodemp.dll"
		} else if let Ok("macos") = std::env::var("CARGO_CFG_TARGET_OS").as_deref() {
			"libcodemp.dylib"
		} else {
			"libcodemp.so"
		};

		let lib_path = std::path::Path::new(&out_dir)
			.parent()
			.unwrap()
			.parent()
			.unwrap()
			.parent()
			.unwrap()
			.join(dylib);
		if !lib_path.exists() {
			println!(
				"cargo:warning=skipping python postprocess, cdylib missing at {}",
				lib_path.display()
			);
			return;
		}

		let module_hints = pyo3_introspection::introspect_cdylib(lib_path, "codemplib")
			.expect("could not extract embedded type hints.");
		let output = pyo3_introspection::module_stub_files(&module_hints);
		println!("{output:?}");

		let root_dir = std::env::var("CARGO_MANIFEST_DIR").expect("unreachable");
		let pydist = std::path::Path::new(&root_dir)
			.join("dist")
			.join("py")
			.join("src")
			.join("autocodemp");

		let pyicontent = output
			.get_key_value(std::path::Path::new("__init__.pyi"))
			.unwrap()
			.1;

		let pyinit = "\
from .codemplib import *

__doc__ = codemplib.__doc__
if hasattr(codemplib, '__all__'):
__all__ = codemplib.__all__";

		let _ = std::fs::write(pydist.join("__init__.py"), pyinit);
		let _ = std::fs::write(pydist.join("codemplib.pyi"), pyicontent);
	}

	#[cfg(feature = "lua")]
	{
		if let Ok("macos") = std::env::var("CARGO_CFG_TARGET_OS").as_deref() {
			println!("cargo:rustc-cdylib-link-arg=-undefined");
			println!("cargo:rustc-cdylib-link-arg=dynamic_lookup");
		}
	}
}
