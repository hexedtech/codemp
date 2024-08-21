use codemp_proto::workspace::workspace_event::Event as WorkspaceEventInner;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "python", pyo3::pyclass)]
pub enum Event {
	FileTreeUpdated(String),
	UserJoin(String),
	UserLeave(String),
}

impl From<WorkspaceEventInner> for Event {
	fn from(event: WorkspaceEventInner) -> Self {
		match event {
			WorkspaceEventInner::Join(e) => Self::UserJoin(e.user.name),
			WorkspaceEventInner::Leave(e) => Self::UserLeave(e.user.name),
			WorkspaceEventInner::Create(e) => Self::FileTreeUpdated(e.path),
			WorkspaceEventInner::Delete(e) => Self::FileTreeUpdated(e.path),
			WorkspaceEventInner::Rename(e) => Self::FileTreeUpdated(e.after),
		}
	}
}

impl From<&WorkspaceEventInner> for Event {
	fn from(event: &WorkspaceEventInner) -> Self {
		Self::from(event.clone())
	}
}
