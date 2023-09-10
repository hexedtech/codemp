//! ### factory
//!
//! a helper trait that any string container can implement, which generates opseqs
//!
//! an OperationFactory trait implementation is provided for String, but plugin developers 
//! should implement their own operation factory interfacing directly with the editor 
//! buffer when possible.

use std::ops::Range;

use operational_transform::{OperationSeq, Operation};
use similar::{TextDiff, ChangeTag};

/// calculate leading no-ops in given opseq
pub const fn leading_noop(seq: &[Operation]) -> u64 { count_noop(seq.first()) }

/// calculate tailing no-ops in given opseq
pub const fn tailing_noop(seq: &[Operation]) -> u64 { count_noop(seq.last())  }

const fn count_noop(op: Option<&Operation>) -> u64 {
	match op {
		None => 0,
		Some(Operation::Retain(n)) => *n,
		Some(_) => 0,
	}
}

/// return the range on which the operation seq is actually applying its changes
pub fn op_effective_range(op: &OperationSeq) -> Range<u64> {
	let first = leading_noop(op.ops());
	let last = op.base_len() as u64 - tailing_noop(op.ops());
	first..last
}

/// a helper trait that any string container can implement, which generates opseqs
///
/// all operations are to be considered mutating current state, obtainable with
/// [OperationFactory::content]. generating an operation has no effect on internal state
///
/// ### examples
///
/// ```rust
/// use codemp::api::OperationFactory;
///
/// let mut factory = String::new();
/// let operation = factory.ins("asd", 0);
/// factory = operation.apply(&factory)?;
/// assert_eq!(factory, "asd");
/// # Ok::<(), codemp::ot::OTError>(())
/// ```
///
/// use [OperationFactory::ins] to add new characters at a specific index
///
/// ```rust
/// # use codemp::api::OperationFactory;
/// # let mut factory = String::from("asd");
/// factory = factory.ins(" dsa", 3).apply(&factory)?;
/// assert_eq!(factory, "asd dsa");
/// # Ok::<(), codemp::ot::OTError>(())
/// ```
///
/// use [OperationFactory::diff] to arbitrarily change text at any position
///
/// ```rust
/// # use codemp::api::OperationFactory;
/// # let mut factory = String::from("asd dsa");
/// factory = factory
///   .diff(2, " xxx ", 5)
///   .expect("replaced region is equal to origin")
///   .apply(&factory)?;
/// assert_eq!(factory, "as xxx sa");
/// # Ok::<(), codemp::ot::OTError>(())
/// ```
///
/// use [OperationFactory::del] to remove characters from given index
///
/// ```rust
/// # use codemp::api::OperationFactory;
/// # let mut factory = String::from("as xxx sa");
/// factory = factory.del(2, 5).apply(&factory)?;
/// assert_eq!(factory, "assa");
/// # Ok::<(), codemp::ot::OTError>(())
/// ```
///
/// use [OperationFactory::replace] to completely replace buffer content
///
/// ```rust
/// # use codemp::api::OperationFactory;
/// # let mut factory = String::from("assa");
/// factory = factory.replace("from scratch")
///   .expect("replace is equal to origin")
///   .apply(&factory)?;
/// assert_eq!(factory, "from scratch");
/// # Ok::<(), codemp::ot::OTError>(())
/// ```
///
/// use [OperationFactory::canc] to remove characters at index, but backwards
///
/// ```rust
/// # use codemp::api::OperationFactory;
/// # let mut factory = String::from("from scratch");
/// factory = factory.canc(12, 8).apply(&factory)?;
/// assert_eq!(factory, "from");
/// # Ok::<(), codemp::ot::OTError>(())
/// ```
pub trait OperationFactory {
	/// the current content of the buffer
	fn content(&self) -> String;

	/// completely replace the buffer with given text
	fn replace(&self, txt: &str) -> Option<OperationSeq> {
		self.diff(0, txt, self.content().len())
	}

	/// transform buffer in range [start..end] with given text
	fn diff(&self, start: usize, txt: &str, end: usize) -> Option<OperationSeq> {
		let mut out = OperationSeq::default();
		let content = self.content();
		let tail_skip = content.len() - end; // TODO len is number of bytes, not chars
		let content_slice = &content[start..end];

		if content_slice == txt {
			// if slice equals given text, no operation should be taken
			return None;
		}

		out.retain(start as u64);

		let diff = TextDiff::from_chars(content_slice, txt);

		for change in diff.iter_all_changes() {
			match change.tag() {
				ChangeTag::Equal => out.retain(1),
				ChangeTag::Delete => out.delete(1),
				ChangeTag::Insert => out.insert(change.value()),
			}
		}

		out.retain(tail_skip as u64);

		Some(out)
	}

	/// insert given chars at target position
	fn ins(&self, txt: &str, pos: u64) -> OperationSeq {
		let mut out = OperationSeq::default();
		let total = self.content().len() as u64;
		out.retain(pos);
		out.insert(txt);
		out.retain(total - pos);
		out
	}

	/// delete n characters forward at given position
	fn del(&self, pos: u64, count: u64) -> OperationSeq {
		let mut out = OperationSeq::default();
		let len = self.content().len() as u64;
		out.retain(pos);
		out.delete(count);
		out.retain(len - (pos+count));
		out
	}

	/// delete n characters backwards at given position
	fn canc(&self, pos: u64, count: u64) -> OperationSeq {
		let mut out = OperationSeq::default();
		let len = self.content().len() as u64;
		out.retain(pos - count);
		out.delete(count);
		out.retain(len - pos);
		out
	}
}

impl OperationFactory for String {
	fn content(&self) -> String {
		self.clone()
	}
}
