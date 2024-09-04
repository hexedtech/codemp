//! # TextChange
//! A high-level representation of a change within a given buffer.

/// An editor-friendly representation of a text change in a given buffer.
/// 
/// It's expressed with a range of characters and a string of content that should replace them,
/// allowing representation of any combination of deletions, insertions or replacements.
///
/// Bulky and large operations will result in a single [`TextChange`] effectively sending the whole
/// new buffer, but smaller changes are efficient and easy to create or apply.
///
/// [`TextChange`] contains an optional `hash` field. This is used for error correction: if
/// provided, it should match the hash of the buffer content **after** applying this change.
/// Note that the `hash` field will not necessarily be provided every time.
///
/// ### Examples
/// To insert 'a' after 4th character we should send a.
///     `TextChange { start: 4, end: 4, content: "a".into(), hash: None }`
///
/// To delete a the fourth character we should send a.
///     `TextChange { start: 3, end: 4, content: "".into(), hash: None }`
///
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "js", napi_derive::napi(object))]
#[cfg_attr(feature = "python", pyo3::pyclass(get_all))]
pub struct TextChange {
	/// Range start of text change, as char indexes in buffer previous state.
	pub start: u32,
	/// Range end of text change, as char indexes in buffer previous state.
	pub end: u32,
	/// New content of text inside span.
	pub content: String,
	/// Optional content hash after applying this change.
	pub hash: Option<i64>,
}

impl TextChange {
	/// Returns the [`std::ops::Range`] representing this change's span.
	pub fn span(&self) -> std::ops::Range<usize> {
		self.start as usize..self.end as usize
	}
}

#[cfg_attr(feature = "python", pyo3::pymethods)]
impl TextChange {
	/// Returns true if this [`TextChange`] deletes existing text.
	///
	/// Note that this is is **not** mutually exclusive with [TextChange::is_insert].
	pub fn is_delete(&self) -> bool {
		self.start < self.end
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
		let pre_index = std::cmp::min(self.start as usize, txt.len());
		let pre = txt.get(..pre_index).unwrap_or("").to_string();
		let post = txt.get(self.end as usize..).unwrap_or("").to_string();
		format!("{}{}{}", pre, self.content, post)
	}
}

#[cfg(test)]
mod tests {
	#[test]
	fn textchange_apply_works_for_insertions() {
		let change = super::TextChange {
			start: 5,
			end: 5,
			content: " cruel".to_string(),
			hash: None,
		};
		let result = change.apply("hello world!");
		assert_eq!(result, "hello cruel world!");
	}

	#[test]
	fn textchange_apply_works_for_deletions() {
		let change = super::TextChange {
			start: 5,
			end: 11,
			content: "".to_string(),
			hash: None,
		};
		let result = change.apply("hello cruel world!");
		assert_eq!(result, "hello world!");
	}

	#[test]
	fn textchange_apply_works_for_replacements() {
		let change = super::TextChange {
			start: 5,
			end: 11,
			content: " not very pleasant".to_string(),
			hash: None,
		};
		let result = change.apply("hello cruel world!");
		assert_eq!(result, "hello not very pleasant world!");
	}

	#[test]
	fn textchange_apply_never_panics() {
		let change = super::TextChange {
			start: 100,
			end: 110,
			content: "a very long string \n which totally matters".to_string(),
			hash: None,
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
			start: 42,
			end: 42,
			content: "".to_string(),
			hash: None,
		};
		let result = change.apply("some important text");
		assert_eq!(result, "some important text");
	}
}
