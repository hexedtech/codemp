#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "js", napi_derive::napi(object))]
#[cfg_attr(feature = "py", pyo3::pyclass(get_all, set_all))]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct Node {
	pub id: String,
	pub name: String,
	pub children: Vec<Node>,
	pub directory: bool,
	pub managed: bool,
}

pub fn treefy(mut codemp_paths: Vec<String>) -> Node {
	let mut root = Node {
		id: "".to_string(), // TODO ??
		name: "".to_string(),
		children: Vec::new(),
		directory: true,
		managed: false,
	};

	codemp_paths.sort();
	
	for path in codemp_paths {
		treefy_rec(&mut root, &path);
	}
	
	root
}

fn treefy_rec(node: &mut Node, path: &str) {
	if let Some(slash_pos) = path.find('/') {
		let node_mut = if let Some(child_node) = node.children.iter_mut().filter(|x| x.name == path[slash_pos..]).next() {
			child_node
		} else {
			let end_segment = path[slash_pos..].find('/').unwrap_or(path.len());
			node.children.push(Node {
				id: "".to_string(),
				name: path[slash_pos..end_segment].to_string(), // TODO: is this legal?
				children: Vec::new(),
				directory: true,
				managed: false
			});
			node.children.last_mut().expect("node not found but it should have just been created!")
		};

		treefy_rec(node_mut, &path[slash_pos..]);
	} else {
		node.children.push(Node {
			id: "".to_string(),
			name: path.to_string(),
			children: Vec::new(),
			directory: false,
			managed: true
		});
	}
}
