pub mod factory;
pub mod processor;

use std::ops::Range;

use operational_transform::{Operation, OperationSeq};
pub use processor::{OperationController, OperationProcessor};
pub use factory::OperationFactory;

pub const fn leading_noop(seq: &[Operation]) -> u64 { count_noop(seq.first()) }
pub const fn tailing_noop(seq: &[Operation]) -> u64 { count_noop(seq.last())  }

const fn count_noop(op: Option<&Operation>) -> u64 {
	match op {
		None => 0,
		Some(op) => match op {
			Operation::Retain(n) => *n,
			_ => 0,
		}
	}
}

pub fn op_effective_range(op: &OperationSeq) -> Range<u64> {
	let first = leading_noop(op.ops());
	let last = op.base_len() as u64 - tailing_noop(op.ops());
	first..last
}
