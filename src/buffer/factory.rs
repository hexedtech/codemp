//! ### factory
//! 
//! a helper trait to produce Operation Sequences, knowing the current 
//! state of the buffer
//!
//! ```rust
//! use codemp::buffer::factory::{OperationFactory, SimpleOperationFactory};
//!
//! let mut factory = SimpleOperationFactory::from("");
//! let op = factory.insert("asd", 0);
//! factory.update(op)?;
//! assert_eq!(factory.content(), "asd");
//! # Ok::<(), codemp::ot::OTError>(())
//! ```
//!
//! use [OperationFactory::insert] to add new characters at a specific index
//!
//! ```rust
//! # use codemp::buffer::factory::{OperationFactory, SimpleOperationFactory};
//! # let mut factory = SimpleOperationFactory::from("asd");
//! factory.update(factory.insert(" dsa", 3))?;
//! assert_eq!(factory.content(), "asd dsa");
//! # Ok::<(), codemp::ot::OTError>(())
//! ```
//!
//! use [OperationFactory::delta] to arbitrarily change text at any position
//!
//! ```rust
//! # use codemp::buffer::factory::{OperationFactory, SimpleOperationFactory};
//! # let mut factory = SimpleOperationFactory::from("asd dsa");
//! let op = factory.delta(2, " xxx ", 5).expect("replaced region is equal to origin");
//! factory.update(op)?;
//! assert_eq!(factory.content(), "as xxx sa");
//! # Ok::<(), codemp::ot::OTError>(())
//! ```
//!
//! use [OperationFactory::delete] to remove characters from given index
//!
//! ```rust
//! # use codemp::buffer::factory::{OperationFactory, SimpleOperationFactory};
//! # let mut factory = SimpleOperationFactory::from("as xxx sa");
//! factory.update(factory.delete(2, 5))?;
//! assert_eq!(factory.content(), "assa");
//! # Ok::<(), codemp::ot::OTError>(())
//! ```
//!
//! use [OperationFactory::replace] to completely replace buffer content
//!
//! ```rust
//! # use codemp::buffer::factory::{OperationFactory, SimpleOperationFactory};
//! # let mut factory = SimpleOperationFactory::from("assa");
//! let op = factory.replace("from scratch").expect("replace is equal to origin");
//! factory.update(op)?;
//! assert_eq!(factory.content(), "from scratch");
//! # Ok::<(), codemp::ot::OTError>(())
//! ```
//!
//! use [OperationFactory::cancel] to remove characters at index, but backwards
//!
//! ```rust
//! # use codemp::buffer::factory::{OperationFactory, SimpleOperationFactory};
//! # let mut factory = SimpleOperationFactory::from("from scratch");
//! factory.update(factory.cancel(12, 8))?;
//! assert_eq!(factory.content(), "from");
//! # Ok::<(), codemp::ot::OTError>(())
//! ```
//!

use std::ops::Range;

use operational_transform::{OperationSeq, Operation, OTError};
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
pub trait OperationFactory {
	/// the current content of the buffer
	fn content(&self) -> String;

	/// completely replace the buffer with given text
	fn replace(&self, txt: &str) -> Option<OperationSeq> {
		self.delta(0, txt, self.content().len())
	}

	/// transform buffer in range [start..end] with given text
	fn delta(&self, start: usize, txt: &str, end: usize) -> Option<OperationSeq> {
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
	fn insert(&self, txt: &str, pos: u64) -> OperationSeq {
		let mut out = OperationSeq::default();
		let total = self.content().len() as u64;
		out.retain(pos);
		out.insert(txt);
		out.retain(total - pos);
		out
	}

	/// delete n characters forward at given position
	fn delete(&self, pos: u64, count: u64) -> OperationSeq {
		let mut out = OperationSeq::default();
		let len = self.content().len() as u64;
		out.retain(pos);
		out.delete(count);
		out.retain(len - (pos+count));
		out
	}

	/// delete n characters backwards at given position
	fn cancel(&self, pos: u64, count: u64) -> OperationSeq {
		let mut out = OperationSeq::default();
		let len = self.content().len() as u64;
		out.retain(pos - count);
		out.delete(count);
		out.retain(len - pos);
		out
	}
}

/// a simple example operation factory that just wraps a string
pub struct SimpleOperationFactory(String);

impl SimpleOperationFactory {
	/// applies given OperationSeq to underlying string, changing it
	pub fn update(&mut self, op: OperationSeq) -> Result<(), OTError> {
		self.0 = op.apply(&self.0)?;
		Ok(())
	}
}

impl From::<&str> for SimpleOperationFactory {
	fn from(value: &str) -> Self {
		SimpleOperationFactory(value.to_string())
	}
}

impl OperationFactory for SimpleOperationFactory {
	fn content(&self) -> String {
		self.0.clone()
	}
}
