// Must be clonable, containing references to the actual state maybe? Or maybe give everyone an Arc, idk
#[derive(Debug)]
pub struct Workspace {
	pub name: String,
	pub content: String,
}

impl Workspace {
	pub fn new(name: String, content: String) -> Self {
		Workspace { name , content }
	}
}

impl Default for Workspace {
	fn default() -> Self {
		Workspace { name: "fuck you".to_string() , content: "too".to_string() }
	}
}
