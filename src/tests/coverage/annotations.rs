#[test]
#[cfg(all(test, feature = "lua"))]
fn lua_annotations_should_cover_ffi_api_surface() {
	let annotations = include_str!("../../../dist/lua/annotations.lua");

	let source = concat!(
		include_str!("../../ffi/lua/client.rs"),
		include_str!("../../ffi/lua/workspace.rs"),
		include_str!("../../ffi/lua/buffer.rs"),
		include_str!("../../ffi/lua/cursor.rs"),
	);

	let re = regex::Regex::new("add_method\\(\\s+\"(\\w+)\",").expect("failed building regex");

	let mut missing = Vec::new();
	for (_, [fn_name]) in re.captures_iter(source).map(|c| c.extract()) {
		if !annotations.contains(fn_name) {
			#[cfg(feature = "ci")]
			{
				println!("::warning title=Coverage::Missing Lua annotations for method: '{fn_name}'");
			}
			missing.push(fn_name.to_string());
		}
	}

	assert!(
		missing.is_empty(),
		"missing lua annotations for methods: '{}'",
		missing.join("', '"),
	);
}
