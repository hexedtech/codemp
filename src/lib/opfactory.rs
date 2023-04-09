use operational_transform::{OperationSeq, OTError};


#[derive(Clone)]
pub struct OperationFactory {
	content: String,
}

impl OperationFactory {
	pub fn new(init: Option<String>) -> Self {
		OperationFactory { content: init.unwrap_or(String::new()) }
	}

	pub fn check(&self, txt: &str) -> bool {
		self.content == txt
	}

	pub fn replace(&mut self, txt: &str) -> OperationSeq {
		let out = OperationSeq::default();
		if self.content == txt {
			return out; // nothing to do
		}

		todo!()
	}

	pub fn insert(&mut self, txt: &str, pos: u64) -> Result<OperationSeq, OTError> {
		let mut out = OperationSeq::default();
		out.retain(pos);
		out.insert(txt);
		self.content = out.apply(&self.content)?; // TODO does aplying mutate the OpSeq itself?
		Ok(out)
	}

	pub fn delete(&mut self, pos: u64, count: u64) -> Result<OperationSeq, OTError> {
		let mut out = OperationSeq::default();
		out.retain(pos - count);
		out.delete(count);
		self.content = out.apply(&self.content)?; // TODO does aplying mutate the OpSeq itself?
		Ok(out)
	}

	pub fn cancel(&mut self, pos: u64, count: u64) -> Result<OperationSeq, OTError> {
		let mut out = OperationSeq::default();
		out.retain(pos);
		out.delete(count);
		self.content = out.apply(&self.content)?; // TODO does aplying mutate the OpSeq itself?
		Ok(out)
	}

	pub fn process(&mut self, op: OperationSeq) -> Result<(), OTError> {
		self.content = op.apply(&self.content)?;
		Ok(())
	}

}
