use std::ops::Range;

use operational_transform::{OperationSeq, OTError};

use crate::operation::factory::OperationFactory;

pub trait OperationProcessor : OperationFactory {
	fn apply(&self, op: OperationSeq) -> Result<Range<u64>, OTError>;
	fn process(&self, op: OperationSeq) -> Result<Range<u64>, OTError>;
}
