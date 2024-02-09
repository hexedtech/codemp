//! # TextChange
//!
//! an editor-friendly representation of a text change in a buffer
//! to easily interface with codemp from various editors

#[cfg(feature = "woot")]
use crate::woot::{WootResult, woot::Woot, crdt::{TextEditor, CRDT, Op}};

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
pub struct TextChange {
	/// range of text change, as char indexes in buffer previous state
	pub span: std::ops::Range<usize>,
	/// new content of text inside span
	pub content: String,
}

impl TextChange {
	#[cfg(feature = "woot")]
	/// create a new TextChange from the difference of given strings
	pub fn from_diff(before: &str, after: &str) -> TextChange {
		let diff = similar::TextDiff::from_chars(before, after);
		let mut start = 0;
		let mut end = 0;
		let mut from_beginning = true;
		for op in diff.ops() {
			match op {
				similar::DiffOp::Equal { len, .. } => {
					if from_beginning {
						start += len
					} else {
						end += len
					}
				},
				_ => {
					end = 0;
					from_beginning = false;
				}
			}
		}
		let end_before = before.len() - end;
		let end_after = after.len() - end;

		TextChange {
			span: start..end_before,
			content: after[start..end_after].to_string(),
		}
	}

	#[cfg(feature = "woot")]
	/// consume the [TextChange], transforming it into a Vec of [woot::crdt::Op]
	pub fn transform(self, woot: &Woot) -> WootResult<Vec<Op>> {
		let mut out = Vec::new();
		if self.is_empty() { return Ok(out); } // no-op
		let view = woot.view();
		let Some(span) = view.get(self.span.clone()) else {
			return Err(crate::woot::WootError::OutOfBounds);
		};
		let diff = similar::TextDiff::from_chars(span, &self.content);
		for (i, diff) in diff.iter_all_changes().enumerate() {
			match diff.tag() {
				similar::ChangeTag::Equal => {},
				similar::ChangeTag::Delete => match woot.delete_one(self.span.start + i) {
					Err(e) => tracing::error!("could not create deletion: {}", e),
					Ok(op) => out.push(op),
				},
				similar::ChangeTag::Insert => {
					match woot.insert(self.span.start + i, diff.value()) {
						Ok(mut op) => out.append(&mut op),
						Err(e) => tracing::error!("could not create insertion: {}", e),
					}
				},
			}
		}
		Ok(out)
	}

	/// returns true if this TextChange deletes existing text
	pub fn is_deletion(&self) -> bool {
		!self.span.is_empty()
	}

	/// returns true if this TextChange adds new text
	pub fn is_addition(&self) -> bool {
		!self.content.is_empty()
	}

	/// returns true if this TextChange is effectively as no-op
	pub fn is_empty(&self) -> bool {
		!self.is_deletion() && !self.is_addition()
	}

	/// applies this text change to given text, returning a new string
	pub fn apply(&self, txt: &str) -> String {
		let pre_index = std::cmp::min(self.span.start, txt.len());
		let pre = txt.get(..pre_index).unwrap_or("").to_string();
		let post = txt.get(self.span.end..).unwrap_or("").to_string();
		format!("{}{}{}", pre, self.content, post)
	}

	/// convert from byte index to row and column
	/// txt must be the whole content of the buffer, in order to count lines
	#[cfg(feature = "proto")]
	pub fn index_to_rowcol(txt: &str, index: usize) -> crate::proto::cursor::RowCol {
		// FIXME might panic, use .get()
		let row = txt[..index].matches('\n').count() as i32;
		let col = txt[..index].split('\n').last().unwrap_or("").len() as i32;
		crate::proto::cursor::RowCol { row, col }
	}
}

#[cfg(test)]
mod tests {
	#[test]
	fn textchange_diff_works_for_deletions() {
		let change = super::TextChange::from_diff(
			"sphinx of black quartz, judge my vow",
			"sphinx of quartz, judge my vow"
		);
		assert_eq!(change.span, 10..16);
		assert_eq!(change.content, "");
	}

	#[test]
	fn textchange_diff_works_for_insertions() {
		let change = super::TextChange::from_diff(
			"sphinx of quartz, judge my vow",
			"sphinx of black quartz, judge my vow"
		);
		assert_eq!(change.span, 10..10);
		assert_eq!(change.content, "black ");
	}

	#[test]
	fn textchange_diff_works_for_changes() {
		let change = super::TextChange::from_diff(
			"sphinx of black quartz, judge my vow",
			"sphinx who watches the desert, judge my vow"
		);
		assert_eq!(change.span, 7..22);
		assert_eq!(change.content, "who watches the desert");
	}

	#[test]
	fn textchange_apply_works_for_insertions() {
		let change = super::TextChange { span: 5..5, content: " cruel".to_string() };
		let result = change.apply("hello world!");
		assert_eq!(result, "hello cruel world!");
	}

	#[test]
	fn textchange_apply_works_for_deletions() {
		let change = super::TextChange { span: 5..11, content: "".to_string() };
		let result = change.apply("hello cruel world!");
		assert_eq!(result, "hello world!");
	}

	#[test]
	fn textchange_apply_works_for_replacements() {
		let change = super::TextChange { span: 5..11, content: " not very pleasant".to_string() };
		let result = change.apply("hello cruel world!");
		assert_eq!(result, "hello not very pleasant world!");
	}

	#[test]
	fn textchange_apply_never_panics() {
		let change = super::TextChange { span: 100..110, content: "a very long string \n which totally matters".to_string() };
		let result = change.apply("a short text");
		assert_eq!(result, "a short texta very long string \n which totally matters");
	}

	#[test]
	fn empty_diff_produces_empty_textchange() {
		let change = super::TextChange::from_diff("same \n\n text", "same \n\n text");
		assert!(change.is_empty());
	}
	
	#[test]
	fn empty_textchange_doesnt_alter_buffer() {
		let change = super::TextChange { span: 42..42, content: "".to_string() };
		let result = change.apply("some important text");
		assert_eq!(result, "some important text");
	}
}
