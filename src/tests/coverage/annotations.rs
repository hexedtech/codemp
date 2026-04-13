#[test]
#[cfg(test)]
fn lua_annotations_should_cover_ffi_api_surface() {
	let annotations = include_str!("../../../dist/lua/annotations.lua");

	let source_maps = [
		("Session", include_str!("../../ffi/lua/session.rs"), false),
		("Workspace", include_str!("../../ffi/lua/workspace.rs"), false),
		("BufferController", include_str!("../../ffi/lua/buffer.rs"), false),
		("CursorController", include_str!("../../ffi/lua/cursor.rs"), false),
		("Codemp", include_str!("../../ffi/lua/mod.rs"), true),
	];

	let re = regex::Regex::new("add_method\\(\\s*\"(\\w+)\",|exports\\.set\\(\\s+\"(\\w+)\",").expect("failed building regex");

	let mut missing = Vec::new();
	for (clazz, source, is_static) in source_maps {
		for (_, [fn_name]) in re.captures_iter(source).map(|c| c.extract()) {
			let sep = if is_static { '.' } else { ':' };
			let search = format!("{clazz}{sep}{fn_name}");
			if !annotations.contains(&search) {
				#[cfg(feature = "ci")]
				{
					println!("::warning title=Coverage::Missing Lua annotations for method: '{search}'");
				}
				missing.push(search);
			}
		}
	}

	assert!(
		missing.is_empty(),
		"missing lua annotations for methods: '{}'",
		missing.join("', '"),
	);
}

#[test]
#[cfg(test)]
fn java_annotations_should_cover_ffi_api_surface() {

	let mut annotations_map = std::collections::HashMap::new();
	for (clazz, content) in [
		("Session", include_str!("../../../dist/java/src/mp/code/Session.java")),
		("Workspace", include_str!("../../../dist/java/src/mp/code/Workspace.java")),
		("Extensions", include_str!("../../../dist/java/src/mp/code/Extensions.java")),
		("BufferController", include_str!("../../../dist/java/src/mp/code/BufferController.java")),
		("CursorController", include_str!("../../../dist/java/src/mp/code/CursorController.java")),
	] {
		annotations_map.insert(clazz, content);
	}

	let source = concat!(
		include_str!("../../ffi/java/session.rs"),
		include_str!("../../ffi/java/workspace.rs"),
		include_str!("../../ffi/java/buffer.rs"),
		include_str!("../../ffi/java/cursor.rs"),
		include_str!("../../ffi/java/ext.rs"),
	);

	let re = regex::Regex::new("#\\[jni\\(.*class = \"(\\w+)\".*\\)\\]\\nfn (\\w+)\\(").expect("failed building regex");

	let mut missing = Vec::new();
	for (_, [clazz, fn_name]) in re.captures_iter(source).map(|c| c.extract()) {
		if !annotations_map.get(clazz).unwrap_or(&"").contains(fn_name) {
			#[cfg(feature = "ci")]
			{
				println!("::warning title=Coverage::Missing Java annotations for method: '{clazz}.{fn_name}'");
			}
			missing.push(format!("{clazz}.{fn_name}"));
		}
	}

	assert!(
		missing.is_empty(),
		"missing java annotations for methods: '{}'",
		missing.join("', '"),
	);
}
