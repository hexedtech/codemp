pub mod buffer;
pub mod cursor;

use std::ops::Range;

use operational_transform::{Operation, OperationSeq};
use tonic::async_trait;

#[async_trait]
pub trait ControllerWorker<T> {
	fn subscribe(&self) -> T;
	async fn work(self);
}

pub const fn leading_noop(seq: &[Operation]) -> u64 { count_noop(seq.first()) }
pub const fn tailing_noop(seq: &[Operation]) -> u64 { count_noop(seq.last())  }

const fn count_noop(op: Option<&Operation>) -> u64 {
	match op {
		None => 0,
		Some(Operation::Retain(n)) => *n,
		Some(_) => 0,
	}
}

pub fn op_effective_range(op: &OperationSeq) -> Range<u64> {
	let first = leading_noop(op.ops());
	let last = op.base_len() as u64 - tailing_noop(op.ops());
	first..last
}
