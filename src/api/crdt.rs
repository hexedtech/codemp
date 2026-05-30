
pub trait CRDT: Default {
	/// represents current state of this CRDT, for all users
	type Version: Send + Sync + Clone + std::fmt::Debug + Eq + Default;
	/// identifies a location inside the text content
	type Location: Send + Sync + Clone + std::fmt::Debug + Eq + Default;
	/// specific id for an agent applying changes
	// TODO get rid of this?
	type AgentID: Send + Sync + Clone + std::fmt::Debug + Eq;
	/// error returned while integrating remote changes
	type Err: std::error::Error;

	/// data type for a Diff: container type with state changes, which can be shared over network
	type Diff: Send
		+ Sync
		+ std::fmt::Debug
		+ AsRef<[u8]>
		+ TryFrom<Vec<u8>, Error: std::fmt::Debug>;

	/// most recent version for this CRDT
	fn version(&self) -> Self::Version;

	/// translate a name to an agent id
	// TODO get rid of this?
	fn agent(&mut self, agent: impl AsRef<str>) -> Self::AgentID;

	/// generate a diff between current version and given version, which can be integrated at a later time
	fn diff(&self, from: Self::Version) -> Self::Diff;

	/// integrate a diff, merging remote changes. this consumes a previously generated Diff
	fn integrate(&mut self, diff: Self::Diff) -> Result<(), Self::Err>;

	/// get current state of CRDT, as string
	fn view(&self) -> String;

	#[deprecated = "should probably be left to `serde::Serialize`"] // TODO
	fn serialize(&self) -> Vec<u8>;

	/// insert text in CRDT content
	///
	/// if insertion is in-bounds and effectively changes (len > 0) it will return a new version.
	/// (this means that None results are for no-ops and out-of-bounds operations)
	fn insert(
		&mut self,
		agent: Self::AgentID,
		location: Self::Location,
		text: impl AsRef<str>,
	) -> Option<Self::Version>;

	/// delete characters from CRDT text content
	///
	/// if deletion is in-bounds and effectively changes (len > 0) it will return a new version.
	/// (this means that None results are for no-ops and out-of-bounds operations)
	fn delete(
		&mut self,
		agent: Self::AgentID,
		location: Self::Location,
		amount: usize,
	) -> Option<Self::Version>;
}
