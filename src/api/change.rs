//! # TextChange
//! A high-level representation of a change within a given buffer.

/// A [`TextChange`] event happening on a buffer.
///
/// Contains the change itself, the new version after this change and an optional `hash` field.
/// This is used for error correction: if provided, it should match the hash of the buffer
/// content **after** applying this change. Note that the `hash` field will not necessarily
/// be provided every time.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "js", napi_derive::napi(object))]
#[cfg_attr(any(feature = "py", feature = "py-noabi"), pyo3::pyclass(get_all))]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct BufferUpdate {
	/// Optional content hash after applying this change.
	pub hash: Option<i64>,
	/// CRDT version after this change has been applied.
	pub version: Vec<i64>,
	/// The change that has occurred.
	pub change: TextChange,
}

/// An editor-friendly representation of a text change in a given buffer.
///
/// It's expressed with a range of characters and a string of content that should replace them,
/// allowing representation of any combination of deletions, insertions or replacements.
///
/// Bulky and large operations will result in a single [`TextChange`] effectively sending the whole
/// new buffer, but smaller changes are efficient and easy to create or apply.
///
/// ### Examples
/// To insert 'a' after 4th character we should send:
/// ```
/// codemp::api::TextChange { start_idx: 4, end_idx: 4, content: "a".into() };
/// ```
///
/// To delete the fourth character we should send:
/// ```
/// codemp::api::TextChange { start_idx: 3, end_idx: 4, content: "".into() };
/// ```
///
/// ```
/// let change = codemp::api::TextChange {
///   start_idx: 6,
///   end_idx: 11,
///   content: "mom".to_string()
/// };
/// let before = "hello world!";
/// let after = change.apply(before);
/// assert_eq!(after, "hello mom!");
/// ```
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "js", napi_derive::napi(object))]
#[cfg_attr(any(feature = "py", feature = "py-noabi"), pyo3::pyclass(get_all))]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct TextChange {
	/// Range start of text change, as char indexes in buffer previous state.
	pub start_idx: u32,
	/// Range end of text change, as char indexes in buffer previous state.
	pub end_idx: u32,
	/// New content of text inside span.
	pub content: String,
}

impl TextChange {
	/// Returns the [`std::ops::Range`] representing this change's span.
	pub fn span(&self) -> std::ops::Range<usize> {
		self.start_idx as usize..self.end_idx as usize
	}
}

#[cfg_attr(any(feature = "py", feature = "py-noabi"), pyo3::pymethods)]
impl TextChange {
	/// Returns true if this [`TextChange`] deletes existing text.
	///
	/// Note that this is is **not** mutually exclusive with [TextChange::is_insert].
	pub fn is_delete(&self) -> bool {
		self.start_idx < self.end_idx
	}

	/// Returns true if this [`TextChange`] adds new text.
	///
	/// Note that this is is **not** mutually exclusive with [TextChange::is_delete].
	pub fn is_insert(&self) -> bool {
		!self.content.is_empty()
	}

	/// Returns true if this [`TextChange`] is effectively as no-op.
	pub fn is_empty(&self) -> bool {
		!self.is_delete() && !self.is_insert()
	}

	/// Applies this text change to given text, returning a new string.
	pub fn apply(&self, txt: &str) -> String {
		let pre_index = std::cmp::min(self.start_idx as usize, txt.len());
		let pre = txt.get(..pre_index).unwrap_or("").to_string();
		let post = txt.get(self.end_idx as usize..).unwrap_or("").to_string();
		format!("{}{}{}", pre, self.content, post)
	}
}

#[cfg(test)]
mod tests {
	#[test]
	fn textchange_apply_works_for_insertions() {
		let change = super::TextChange {
			start_idx: 5,
			end_idx: 5,
			content: " cruel".to_string(),
		};
		let result = change.apply("hello world!");
		assert_eq!(result, "hello cruel world!");
	}

	#[test]
	fn textchange_apply_works_for_deletions() {
		let change = super::TextChange {
			start_idx: 5,
			end_idx: 11,
			content: "".to_string(),
		};
		let result = change.apply("hello cruel world!");
		assert_eq!(result, "hello world!");
	}

	#[test]
	fn textchange_apply_works_for_replacements() {
		let change = super::TextChange {
			start_idx: 5,
			end_idx: 11,
			content: " not very pleasant".to_string(),
		};
		let result = change.apply("hello cruel world!");
		assert_eq!(result, "hello not very pleasant world!");
	}

	#[test]
	fn textchange_apply_never_panics() {
		let change = super::TextChange {
			start_idx: 100,
			end_idx: 110,
			content: "a very long string \n which totally matters".to_string(),
		};
		let result = change.apply("a short text");
		assert_eq!(
			result,
			"a short texta very long string \n which totally matters"
		);
	}

	#[test]
	fn empty_textchange_doesnt_alter_buffer() {
		let change = super::TextChange {
			start_idx: 42,
			end_idx: 42,
			content: "".to_string(),
		};
		let result = change.apply("some important text");
		assert_eq!(result, "some important text");
	}
}
