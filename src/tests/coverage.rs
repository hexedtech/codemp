#![allow(missing_docs)] // internal test helper

use std::collections::{BTreeMap, BTreeSet};
use std::fs;

fn parse(path: &str) -> syn::File {
	syn::parse_file(&fs::read_to_string(path).expect("Could not parse file")).unwrap()
}

// 1) Discover core API from target objects
fn discover_core_surface(files: &[&str], targets: &[&str]) -> BTreeMap<String, BTreeSet<String>> {
	let mut out_targets: BTreeMap<String, BTreeSet<String>> = targets
		.iter()
		.map(|t| (t.to_string(), BTreeSet::new()))
		.collect();

	let mut supertraits_to_check: BTreeMap<String, String> = BTreeMap::new();

	for file in files {
		let ast = parse(file);

		for item in ast.items {
			match item {
				syn::Item::Impl(item_impl) => {
					let ty = match &*item_impl.self_ty {
						syn::Type::Path(p) => p.path.segments.last().unwrap().ident.to_string(),
						_ => continue,
					};

					if !out_targets.contains_key(&ty) {
						continue;
					}

					if item_impl.trait_.is_none() {
						for it in item_impl.items {
							if let syn::ImplItem::Fn(f) = it {
								if matches!(f.vis, syn::Visibility::Public(_)) {
									out_targets
										.get_mut(&ty)
										.unwrap()
										.insert(f.sig.ident.to_string());
								}
							}
						}
					}
				}

				syn::Item::Trait(item_trait) => {
					let trait_name = item_trait.ident.to_string();
					if !out_targets.contains_key(&trait_name) {
						continue;
					}

					if !item_trait.supertraits.is_empty() {
						for tb in item_trait.supertraits {
							// we save the supertypes to check also those if present in a second pass.
							if let syn::TypeParamBound::Trait(trait_bound) = tb {
								supertraits_to_check.insert(
									trait_bound.path.segments.last().unwrap().ident.to_string(),
									trait_name.clone(),
								);
							}
						}
					}

					// if our trait has any defined method that needs to be implemented add it.
					if !item_trait.items.is_empty() {
						for it in item_trait.items {
							if let syn::TraitItem::Fn(f) = it {
								// only add the function that don't have a default body.
								if f.default.is_none() {
									out_targets
										.get_mut(&trait_name)
										.unwrap()
										.insert(f.sig.ident.to_string());
								}
							}
						}
					}
				}
				_ => continue,
			}
		}
	}

	// second pass to explore also all supertraits
	for file in files {
		let ast = parse(file);

		for item in ast.items {
			match item {
				syn::Item::Trait(item_trait) => {
					let trait_name = item_trait.ident.to_string();
					if !supertraits_to_check.contains_key(&trait_name) {
						continue;
					}

					// if our trait has any defined method that needs to be implemented add it
					// (to the already existing trait for which this trait is a supertype)
					if !item_trait.items.is_empty() {
						let target_trait = supertraits_to_check.get(&trait_name).unwrap();

						for it in item_trait.items {
							if let syn::TraitItem::Fn(f) = it {
								// only add the function that don't have a default body.
								if f.default.is_none() {
									out_targets
										.get_mut(target_trait)
										.unwrap()
										.insert(f.sig.ident.to_string());
								}
							}
						}
					}
				}
				_ => continue,
			}
		}
	}

	out_targets
}

// search in the ffi source file to see which methods are present or not. very crude.
fn missing_methods(ffi_src: &str, required: BTreeSet<String>) -> Vec<String> {
	required
		.iter()
		.filter(|method| !ffi_src.contains(&format!(".{}(", method)))
		.map(|method| (*method).to_string())
		.collect()
}

// build a "report" of which required methods are missing from the lang ffi.
fn missing_lang_coverage(
	lang: &str,
	ffi_src: &str,
	required: BTreeMap<String, BTreeSet<String>>,
	ignore: &[&str],
) -> Vec<String> {
	let mut missing = BTreeSet::new();
	for (target, methods) in required {
		for method in missing_methods(ffi_src, methods) {
			let full_name = format!("{target}.{method}");
			if ignore.contains(&full_name.as_str()) {
				continue;
			}

			missing.insert(full_name);
		}
	}

	if missing.is_empty() {
		Vec::new()
	} else {
		vec![format!(
			"{lang}: {}",
			missing.into_iter().collect::<Vec<_>>().join(", ")
		)]
	}
}

#[test]
#[cfg(all(test, feature = "py"))]
fn python_ffi_should_cover_rust_api_surface() {
	let targets = &[
		"Client",
		"Workspace",
		"BufferController",
		"CursorController",
		"Controller",
	];

	let files = &[
		"src/client.rs",
		"src/workspace.rs",
		"src/buffer/controller.rs",
		"src/cursor/controller.rs",
		"src/api/controller.rs",
	];

	let required = discover_core_surface(files, targets);

	let python_src = concat!(
		include_str!("../ffi/python/client.rs"),
		include_str!("../ffi/python/workspace.rs"),
		include_str!("../ffi/python/controllers.rs"),
	);

	let python_ignore = ["Client.connect"];

	let missings = missing_lang_coverage("python", python_src, required, &python_ignore);

	assert!(
		missings.is_empty(),
		"missing ffi coverage:\n{}",
		missings.join("\n")
	);
}

#[test]
#[cfg(all(test, feature = "js"))]
fn javascript_ffi_should_cover_rust_api_surface() {
	let targets = &[
		"Client",
		"Workspace",
		"BufferController",
		"CursorController",
		"Controller",
	];

	let files = &[
		"src/client.rs",
		"src/workspace.rs",
		"src/buffer/controller.rs",
		"src/cursor/controller.rs",
		"src/api/controller.rs",
	];

	let required = discover_core_surface(files, targets);

	let js_src = concat!(
		include_str!("../ffi/js/client.rs"),
		include_str!("../ffi/js/workspace.rs"),
		include_str!("../ffi/js/buffer.rs"),
		include_str!("../ffi/js/cursor.rs"),
	);

	let js_ignore = [];

	let missings = missing_lang_coverage("javascript", js_src, required, &js_ignore);

	assert!(
		missings.is_empty(),
		"missing ffi coverage:\n{}",
		missings.join("\n")
	);
}

#[test]
#[cfg(all(test, feature = "lua"))]
fn lua_ffi_should_cover_rust_api_surface() {
	let targets = &[
		"Client",
		"Workspace",
		"BufferController",
		"CursorController",
		"Controller",
	];

	let files = &[
		"src/client.rs",
		"src/workspace.rs",
		"src/buffer/controller.rs",
		"src/cursor/controller.rs",
		"src/api/controller.rs",
	];

	let required = discover_core_surface(files, targets);

	let lua_src = concat!(
		include_str!("../ffi/lua/client.rs"),
		include_str!("../ffi/lua/workspace.rs"),
		include_str!("../ffi/lua/buffer.rs"),
		include_str!("../ffi/lua/cursor.rs"),
	);

	let lua_ignore = ["Client.connect"];

	let missings = missing_lang_coverage("lua", lua_src, required, &lua_ignore);

	assert!(
		missings.is_empty(),
		"missing ffi coverage:\n{}",
		missings.join("\n")
	);
}

#[test]
#[cfg(all(test, feature = "java"))]
fn java_ffi_should_cover_rust_api_surface() {
	let targets = &[
		"Client",
		"Workspace",
		"BufferController",
		"CursorController",
		"Controller",
	];

	let files = &[
		"src/client.rs",
		"src/workspace.rs",
		"src/buffer/controller.rs",
		"src/cursor/controller.rs",
		"src/api/controller.rs",
	];

	let required = discover_core_surface(files, targets);

	let java_src = concat!(
		include_str!("../ffi/java/client.rs"),
		include_str!("../ffi/java/workspace.rs"),
		include_str!("../ffi/java/buffer.rs"),
		include_str!("../ffi/java/cursor.rs"),
	);

	let java_ignore = [];

	let missings = missing_lang_coverage("java", java_src, required, &java_ignore);

	assert!(
		missings.is_empty(),
		"missing ffi coverage:\n{}",
		missings.join("\n")
	);
}
