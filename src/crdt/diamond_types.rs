use diamond_types::list::encoding::ENCODE_PATCH;

#[derive(Default, Debug)]
pub struct DiamondTypesCRDT {
	log: diamond_types::list::OpLog,
}

// TODO fat struct... should split ops (for editor) and data (for server)
#[derive(Debug, Default, Clone)]
pub struct DiamondTypesCRDTDiff {
	pub version: diamond_types::LocalVersion,
	data: Vec<u8>,
	ops: Vec<diamond_types::list::operation::Operation>,
	idx: usize,
}

impl crate::api::CRDT for DiamondTypesCRDT {
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

	fn diff(&self, from: Self::Version) -> Self::Diff {
		let mut out = Vec::new();
		for (_r, op) in self.log.iter_xf_operations_from(&from, &self.log.local_version()) {
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

	fn view(&self) -> String {
		self.log.checkout_tip().content().to_string()
	}

	fn insert(
		&mut self,
		agent: Self::AgentID,
		location: Self::Location,
		text: impl AsRef<str>,
	) -> Option<Self::Version> {
		let _t = self
			.log
			.add_insert(agent, location, text.as_ref());
		Some(self.log.local_version())
	}

	fn delete(
		&mut self,
		agent: Self::AgentID,
		location: Self::Location,
		amount: usize,
	) -> Option<Self::Version> {
		let _t = self
			.log
			.add_delete_without_content(agent, location..location + amount);
		Some(self.log.local_version())
	}

	fn serialize(&self) -> Vec<u8> {
		self.log
			.encode_simple(diamond_types::list::encoding::EncodeOptions::default())
	}
}

impl Iterator for DiamondTypesCRDTDiff {
	type Item = (std::ops::Range<usize>, String);

	fn next(&mut self) -> Option<Self::Item> {
		let out = self.ops.get(self.idx);
		self.idx += 1;
		out.map(|o| match o.kind {
			diamond_types::list::operation::OpKind::Ins => (
				o.loc.span.start..o.loc.span.end,
				o.content_as_str().unwrap_or_default().to_string(),
			),
			diamond_types::list::operation::OpKind::Del => {
				(o.loc.span.start..o.loc.span.end, "".to_string())
			}
		})
	}
}

// TODO it seems the Version type needs to pass FFI boundaries
//      so it cannot be a vague generic. this is an ugly temp fix
#[deprecated = "solve the version problem......"]
pub fn translate_version<T: crate::api::CRDT>(v: T::Version) -> Vec<i64> {
	vec![crate::ext::hash(format!("{v:?}").as_bytes())]
}

#[deprecated = "solve the version problem......"]
pub fn restore_version<T: crate::api::CRDT>(_v: Vec<i64>) -> T::Version {
	todo!()
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
