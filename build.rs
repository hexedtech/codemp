#[cfg(feature = "js")]
extern crate napi_build;

/// The main method of the buildscript, required by some glue modules.
fn main() {
	#[cfg(feature = "java")] {
		let pkg = "com.codemp.jni".to_string();
		let pkg_folder = pkg.replace('.', "/"); // java moment

		let out_dir = std::env::var("OUT_DIR").expect("cargo did not provide OUT_DIR");
		let out_dir = std::path::Path::new(&out_dir);
		let generated_glue_file = out_dir.join("generated_glue.in");
		let src_dir = std::path::Path::new("src")
			.join("glue")
			.join("java");
		let typemap_file = src_dir.join("typemap.in");
		rifgen::Generator::new(rifgen::TypeCases::CamelCase, rifgen::Language::Java, vec![src_dir])
			.generate_interface(&generated_glue_file);

		// build java source path
		let target = out_dir.parent().unwrap().parent().unwrap().parent().unwrap().to_path_buf(); // target/debug

		let mut java_target = target.clone(); // target/debug/java
		java_target.push("java");

		let mut pkg_path = java_target.clone(); // target/debug/java/src/com/codemp/intellij
		pkg_path.push("src");
		pkg_path.push(pkg_folder);

		// delete folder if it exists, then create it
		recreate_path(&pkg_path);

		// generate java code
		let java_cfg = flapigen::JavaConfig::new(pkg_path.clone(), pkg);
		let java_gen = flapigen::Generator::new(flapigen::LanguageConfig::JavaConfig(java_cfg))
			.rustfmt_bindings(true);
		java_gen.expand_many(
			"codemp-intellij",
			&[&generated_glue_file, &typemap_file],
			out_dir.join("glue.rs")
		);

		#[cfg(feature = "java-artifact")] {
			// panic if no jdk
			std::process::Command::new("javac")
				.arg("--version")
				.status()
				.expect("java not found");

			// compile java code
			let mut java_compiled = java_target.clone(); // target/debug/java/classes
			java_compiled.push("classes");
			recreate_path(&java_compiled);

			let mut javac_cmd = std::process::Command::new("javac");
			javac_cmd.arg("-d").arg(java_compiled.as_os_str());
			for java_file in pkg_path.read_dir().unwrap().filter_map(|e| e.ok()) {
				javac_cmd.arg(java_file.path().as_os_str());
			}
			javac_cmd.status().expect("failed to run javac");
	
			// jar it!
			let mut jar_file = target.clone(); // target/debug/codemp-java.jar
			jar_file.push("codemp-java.jar");

			let mut jar_cmd = std::process::Command::new("jar");
			jar_cmd.current_dir(&java_compiled)
				.arg("cf")
				.arg(jar_file.as_os_str());
			for java_file in java_compiled.read_dir().unwrap().filter_map(|e| e.ok()) {
				let relative_path = java_file.path().clone();
				let relative_path = relative_path.strip_prefix(&java_compiled).unwrap();
				jar_cmd.arg(relative_path.as_os_str());
			}
			jar_cmd.status().expect("failed to run jar!");

			println!("cargo:rerun-if-changed={}", generated_glue_file.display());
		}
	}

	#[cfg(feature = "js")] {
		napi_build::setup();
	}
}

#[cfg(feature = "java")]
fn recreate_path(path: &std::path::PathBuf) {
	if path.exists() {
		std::fs::remove_dir_all(path).expect("failed to delete old dir!");
	}
	std::fs::create_dir_all(path).expect("error while creating folder!");
}
