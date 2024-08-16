use codemp_proto::workspace::workspace_event::Event as WorkspaceEventInner;

#[cfg_attr(feature = "python", pyo3::pyclass)]
pub enum Event {
	FileTreeUpdated(String),
	UserJoin(String),
	UserLeave(String),
}

impl From<&WorkspaceEventInner> for Event {
	fn from(event: &WorkspaceEventInner) -> Self {
		match event {
			WorkspaceEventInner::Join(e) => Self::UserJoin(e.user.id.clone()),
			WorkspaceEventInner::Leave(e) => Self::UserLeave(e.user.id.clone()),
			WorkspaceEventInner::Create(e) => Self::FileTreeUpdated(e.path.clone()),
			WorkspaceEventInner::Delete(e) => Self::FileTreeUpdated(e.path.clone()),
			WorkspaceEventInner::Rename(e) => Self::FileTreeUpdated(e.after.clone()),
		}
	}
}
