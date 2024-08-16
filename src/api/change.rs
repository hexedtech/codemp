//! # TextChange
//!
//! an editor-friendly representation of a text change in a buffer
//! to easily interface with codemp from various editors

/// an editor-friendly representation of a text change in a buffer
///
/// this represent a range in the previous state of the string and a new content which should be
/// replaced to it, allowing to represent any combination of deletions, insertions or replacements
///
/// bulk and widespread operations will result in a TextChange effectively sending the whole new
/// buffer, but small changes are efficient and easy to create or apply
///
/// ### examples
/// to insert 'a' after 4th character we should send a
///     `TextChange { span: 4..4, content: "a".into() }`
///
/// to delete a the fourth character we should send a
///     `TextChange { span: 3..4, content: "".into() }`
///
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "js", napi_derive::napi(object))]
#[cfg_attr(feature = "python", pyo3::pyclass(get_all))]
pub struct TextChange {
	/// range start of text change, as char indexes in buffer previous state
	pub start: u32,
	/// range end of text change, as char indexes in buffer previous state
	pub end: u32,
	/// new content of text inside span
	pub content: String,
	/// optional content hash after applying this change
	pub hash: Option<i64>,
}

impl TextChange {
	pub fn span(&self) -> std::ops::Range<usize> {
		self.start as usize..self.end as usize
	}
}

#[cfg_attr(feature = "python", pyo3::pymethods)]
impl TextChange {
	/// returns true if this TextChange deletes existing text
	pub fn is_delete(&self) -> bool {
		self.start < self.end
	}

	/// returns true if this TextChange adds new text
	pub fn is_insert(&self) -> bool {
		!self.content.is_empty()
	}

	/// returns true if this TextChange is effectively as no-op
	pub fn is_empty(&self) -> bool {
		!self.is_delete() && !self.is_insert()
	}

	/// applies this text change to given text, returning a new string
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
