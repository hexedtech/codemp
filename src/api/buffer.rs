//! # Buffer
//! TODO TODO TODO

/// Represents a service buffer
#[derive(Debug, Clone)]
#[cfg_attr(feature = "py", pyo3::pyclass(from_py_object))]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct BufferNode {
	/// Buffer path, sort of like a UNIX path.
	pub path: String,
	/// Wether this buffer gets auto-deleted once all users left
	pub ephemeral: bool,
}

impl From<codemp_proto::files::BufferNode> for BufferNode {
	fn from(value: codemp_proto::files::BufferNode) -> Self {
		Self {
			path: value.path.into(),
			ephemeral: value.ephemeral,
		}
	}
}
