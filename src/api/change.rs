//! # TextChange
//!
//! an editor-friendly representation of a text change in a buffer
//! to easily interface with codemp from various editors

use crate::proto::RowCol;

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

	/// convert from byte index to row and column
	/// txt must be the whole content of the buffer, in order to count lines
	pub fn index_to_rowcol(txt: &str, index: usize) -> RowCol {
		let row = txt[..index].matches('\n').count() as i32;
		let col = txt[..index].split('\n').last().unwrap_or("").len() as i32;
		RowCol { row, col }
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
}
