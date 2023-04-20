use std::ops::Range;

use operational_transform::{OperationSeq, OTError};

use crate::operation::factory::OperationFactory;

#[tonic::async_trait]
pub trait OperationProcessor : OperationFactory {
	async fn apply(&self, op: OperationSeq) -> Result<Range<u64>, OTError>;
	async fn process(&self, op: OperationSeq) -> Result<Range<u64>, OTError>;
}
