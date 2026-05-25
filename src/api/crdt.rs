#![allow(missing_docs)]

use diamond_types::list::encoding::ENCODE_PATCH;

pub trait CRDT: Default {
	type Version: Send + Sync + Clone + std::fmt::Debug + Eq + Ord + Default;
	type AgentID: Send + Sync + Clone + std::fmt::Debug + Eq;
	type Location;

	type Diff : Send + Sync + std::fmt::Debug + AsRef<[u8]> + TryFrom<Vec<u8>, Error: std::fmt::Debug> + Iterator<Item = (std::ops::Range<usize>, String)>; // TODO this should be serializable so it can travel over wire easily
	type Err: std::error::Error;

	fn version(&self) -> Self::Version;

	fn agent(&mut self, agent: impl AsRef<str>) -> Self::AgentID;

	fn diff(&self, from: Self::Version, to: Self::Version) -> Self::Diff;
	fn integrate(&mut self, diff: Self::Diff) -> Result<(), Self::Err>;

	fn view(&self) -> String {
		self.view_at(self.version())
	}

	fn view_at(&self, time: Self::Version) -> String;

	#[deprecated = "should probably be left to `serde::Serialize`"] // TODO
	fn serialize(&self) -> Vec<u8>;

	// utility variations of `op()`

	fn insert(
		&mut self,
		agent: Self::AgentID,
		location: Self::Location,
		text: impl AsRef<str>,
	) -> Result<Self::Version, Self::Err> {
		self.insert_at(agent, location, self.version(), text)
	}

	fn insert_at(
		&mut self,
		agent: Self::AgentID,
		location: Self::Location,
		time: Self::Version,
		text: impl AsRef<str>,
	) -> Result<Self::Version, Self::Err>;

	fn delete(
		&mut self,
		agent: Self::AgentID,
		location: Self::Location,
		amount: usize,
	) -> Result<Self::Version, Self::Err> {
		self.delete_at(agent, location, self.version(), amount)
	}

	fn delete_at(
		&mut self,
		agent: Self::AgentID,
		location: Self::Location,
		time: Self::Version,
		amount: usize,
	) -> Result<Self::Version, Self::Err>;

}

#[derive(Default, Debug)]
pub struct DiamondTypesCRDT {
	log: diamond_types::list::OpLog,
}

// TODO fat struct... should split ops (for editor) and data (for server)
#[derive(Debug, Default)]
pub struct DiamondTypesCRDTDiff {
	pub version: diamond_types::LocalVersion,
	data: Vec<u8>,
	ops: Vec<diamond_types::list::operation::Operation>,
	idx: usize,
}

impl CRDT for DiamondTypesCRDT {
	type Version = diamond_types::LocalVersion;
	type AgentID = diamond_types::AgentId;
	type Location = usize;
	type Err = diamond_types::list::encoding::encode_tools::ParseError;
	type Diff = DiamondTypesCRDTDiff;

	fn version(&self) -> Self::Version {
		self.log.local_version()
	}

	fn agent(&mut self, agent: impl AsRef<str>) -> Self::AgentID {
		self.log.get_or_create_agent_id(agent.as_ref())
	}

	fn diff(&self, from: Self::Version, to: Self::Version) -> Self::Diff {
		let mut out = Vec::new();
		for (_r, op) in self.log.iter_xf_operations_from(&from, &to) {
			// TODO we don't get op agents, which means we lose them here...
			if let Some(op) = op {
				out.push(op);
			}			
		}
		DiamondTypesCRDTDiff {
			version: from.clone(),
			data: self.log.encode_from(ENCODE_PATCH, &from),
			ops: out,
			idx: 0,
		}
	}

	fn integrate(&mut self, diff: Self::Diff) -> Result<(), Self::Err> {
		self.log.decode_and_add(&diff.data)?;
		Ok(())
	}

	fn view_at(&self, time: Self::Version) -> String {
		self.log.checkout(&time).content().to_string()
	}

	fn insert_at(
			&mut self,
			agent: Self::AgentID,
			location: Self::Location,
			time: Self::Version,
			text: impl AsRef<str>,
		) -> Result<Self::Version, Self::Err>
	{
		let _t = self.log.add_insert_at(agent, &time, location, text.as_ref());
		Ok(self.log.local_version())
	}

	fn delete_at(
			&mut self,
			agent: Self::AgentID,
			location: Self::Location,
			time: Self::Version,
			amount: usize,
		) -> Result<Self::Version, Self::Err>
	{
		let _t = self.log.add_delete_at(agent, &time, location..location+amount);
		Ok(self.log.local_version())
	}

	fn serialize(&self) -> Vec<u8> {
		self.log
			.encode_simple(diamond_types::list::encoding::EncodeOptions::default())	}
}

impl Iterator for DiamondTypesCRDTDiff {
	type Item = (std::ops::Range<usize>, String);

	fn next(&mut self) -> Option<Self::Item> {
		let out = self.ops.get(self.idx);
		self.idx += 1;
		out.map(|o| match o.kind {
    	diamond_types::list::operation::OpKind::Ins => {
    		(o.loc.span.start..o.loc.span.end, o.content_as_str().unwrap_or_default().to_string())
    	},
    	diamond_types::list::operation::OpKind::Del => {
    		(o.loc.span.start..o.loc.span.end, "".to_string())
    	},
		})
	}
}


impl AsRef<[u8]> for DiamondTypesCRDTDiff {
	fn as_ref(&self) -> &[u8] {
		&self.data
	}
}

impl TryFrom<Vec<u8>> for DiamondTypesCRDTDiff {
	type Error = std::convert::Infallible;
	fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
		Ok(DiamondTypesCRDTDiff {
			data: value,
			ops: vec![],
			version: diamond_types::LocalVersion::default(),
			idx: 0,
		})
	}
}
