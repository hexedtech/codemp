#[cfg(feature = "js")]
extern crate napi_build;

#[cfg(feature = "python")]
extern crate pyo3_build_config;

/// The main method of the buildscript, required by some glue modules.
fn main() {
	#[cfg(feature = "js")]
	{
		napi_build::setup();
	}

	#[cfg(feature = "python")]
	{
		pyo3_build_config::add_extension_module_link_args();
	}
}
