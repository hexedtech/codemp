use yrs::{GetString, Observable, ReadTxn, Text, Transact, updates::decoder::Decode};

pub struct YrsCRDT {
	doc: yrs::Doc,
	txt: yrs::TextRef,
}

impl Default for YrsCRDT {
	fn default() -> Self {
		let doc = yrs::Doc::new();
		let txt = doc.get_or_insert_text("text");
		txt.observe(|tx, event| {
			for delta in event.delta(&tx) {
				match delta {
					yrs::types::Delta::Inserted(x, hash_map) => {
					},
					yrs::types::Delta::Deleted(x) => {},
					yrs::types::Delta::Retain(x, hash_map) => {},
				}
			}
		});
		Self { doc, txt }
	}
}

impl crate::api::CRDT for YrsCRDT {
	type Version = yrs::StateVector;
	type Location = u32;
	type AgentID = ();
	type Diff = Vec<u8>;
	type Err = yrs::error::Error;

	fn version(&self) -> Self::Version {
		self.doc.transact().state_vector()
	}

	fn agent(&mut self, _agent: impl AsRef<str>) -> Self::AgentID {}

	fn diff(&self, from: Self::Version) -> Self::Diff {
		self.doc.transact().encode_diff_v1(&from)
	}

	fn integrate(&mut self, diff: Self::Diff) -> Result<(), Self::Err> {
		let upd = yrs::Update::decode_v1(&diff)?;
		Ok(self.doc.transact_mut().apply_update(upd)?)
	}

	fn view(&self) -> String {
		self.txt.get_string(&self.doc.transact())
	}

	fn serialize(&self) -> Vec<u8> {
		todo!()
	}

	fn insert(
		&mut self,
		_agent: Self::AgentID,
		location: Self::Location,
		text: impl AsRef<str>,
	) -> Option<Self::Version> {
		// TODO yrs panics if we go out of bounds... check it ourselves!!
		self.txt
			.insert(&mut self.doc.transact_mut(), location, text.as_ref());
		Some(self.doc.transact().state_vector())
	}

	fn delete(
		&mut self,
		_agent: Self::AgentID,
		location: Self::Location,
		amount: usize,
	) -> Option<Self::Version> {
		// TODO yrs panics if we go out of bounds... check it ourselves!!
		self.txt
			.remove_range(&mut self.doc.transact_mut(), location, amount as u32);
		Some(self.doc.transact().state_vector())
	}
}
