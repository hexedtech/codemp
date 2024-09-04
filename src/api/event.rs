//! # Event
//! Real time notification of changes in a workspace, to either users or buffers.
use codemp_proto::workspace::workspace_event::Event as WorkspaceEventInner;

/// Event in a [crate::Workspace].
#[derive(Debug, Clone)]
#[cfg_attr(feature = "python", pyo3::pyclass)]
pub enum Event {
	/// Fired when the file tree changes.
	/// Contains the modified buffer path (deleted, created or renamed).
	FileTreeUpdated(String),
	/// Fired when an user joins the current workspace.
	UserJoin(String),
	/// Fired when an user leaves the current workspace.
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
