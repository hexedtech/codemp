#![allow(missing_docs)]

pub trait CRDT: Default {
	type Version;
	type AgentID : std::fmt::Debug;
	type Snapshot;
	type Err : std::error::Error;

	fn version(&self) -> Self::Version;

	fn agent(&mut self, agent: impl AsRef<str>) -> Self::AgentID;

	fn op(&mut self, agent: Self::AgentID, location: usize, op: Operation) -> Result<Self::Version, Self::Err>;

	fn insert(
		&mut self,
		agent: Self::AgentID,
		location: usize,
		text: impl AsRef<str>,
	) -> Result<Self::Version, Self::Err> {
		self.op(agent, location, Operation::Insert(text.as_ref().to_string()))
	}

	fn delete(
		&mut self,
		agent: Self::AgentID,
		location: usize,
		amount: usize,
	) -> Result<Self::Version, Self::Err> {
		self.op(agent, location, Operation::Delete(amount))
	}

	fn snapshot(&self) -> Self::Snapshot;

	// TODO this should be an inherited `serde::Serialize`
	fn serialize(&self) -> Vec<u8>;
}

// pub struct Operation {
// 	pub agent: usize,
// 	pub position: usize,
// 	pub kind: OperationKind,
// }

pub enum Operation {
	Insert(String),
	Delete(usize),
}

#[deprecated = "this is a placeholder, it shouldn't work like this..."]
pub fn op_from_data(_data: Vec<u8>) -> Operation {
	todo!()
}

#[derive(Default)]
pub struct DiamondTypesCRDT {
	log: diamond_types::list::OpLog,
}

impl CRDT for DiamondTypesCRDT {
	type Version = diamond_types::LocalVersion;
	type AgentID = diamond_types::AgentId;
	type Err = std::convert::Infallible;
	type Snapshot = Vec<u8>;

	fn version(&self) -> Self::Version {
		self.log.local_version()
	}

	fn agent(&mut self, agent: impl AsRef<str>) -> Self::AgentID {
		self.log.get_or_create_agent_id(agent.as_ref())
	}

	fn op(&mut self, agent: Self::AgentID, location: usize, op: Operation) -> Result<Self::Version, Self::Err> {
		match op {
			Operation::Insert(txt) => {
				let _t = self.log.add_insert(agent, location, &txt);
			},
			Operation::Delete(n) => {
				let _t = self.log.add_delete_without_content(agent, location..location + n);
			},
		};

		Ok(self.log.local_version())
	}

	fn snapshot(&self) -> Self::Snapshot {
		self.log.encode_simple(diamond_types::list::encoding::EncodeOptions::default())
	}

	fn serialize(&self) -> Vec<u8> {
		self.snapshot()
	}
}
