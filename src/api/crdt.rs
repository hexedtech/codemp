#![allow(missing_docs)]

pub trait CRDT: Default {
	type Version;

	fn version(&self) -> Self::Version;

	fn register_agent(&mut self, agent: impl AsRef<str>) -> usize;

	fn op(&mut self, op: Operation) -> Result<Self::Version, ()>;
}

pub struct Operation {
	pub agent: usize,
	pub position: usize,
	pub kind: OperationKind,
}

pub enum OperationKind {
	Insert(String),
	Delete(usize),
}

#[derive(Default)]
pub struct FakeCRDT;

impl CRDT for FakeCRDT {
	type Version = usize;

	fn version(&self) -> usize {
		todo!()
	}

	fn register_agent(&mut self, _agent: impl AsRef<str>) -> usize {
		todo!()
	}

	fn op(&mut self, op: Operation) -> Result<Self::Version, ()> {
		todo!()
	}
}

#[deprecated = "lets do this with serde..."]
pub fn encode_op(op: Operation) -> Vec<u8> {
	todo!()
}

#[deprecated = "lets do this with serde..."]
pub fn decode_op(data: Vec<u8>) -> Operation {
	todo!()
}
