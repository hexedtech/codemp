//! # Event
//! Real time notification of changes in a workspace, to either users or buffers.
#![allow(non_upper_case_globals, non_camel_case_types)] // pyo3 fix your shit

use codemp_proto::workspace::workspace_event::Event as WorkspaceEventInner;

/// Event in a [crate::Workspace].
#[derive(Debug, Clone)]
#[cfg_attr(any(feature = "py", feature = "py-noabi"), pyo3::pyclass)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub enum Event {
	/// Fired when the file tree changes.
	/// Contains the modified buffer path (deleted, created or renamed).
	FileTreeUpdated { path: String },
	/// Fired when an user joins the current workspace.
	UserJoin { name: String },
	/// Fired when an user leaves the current workspace.
	UserLeave { name: String },
}

impl From<WorkspaceEventInner> for Event {
	fn from(event: WorkspaceEventInner) -> Self {
		match event {
			WorkspaceEventInner::Join(e) => Self::UserJoin { name: e.user.name },
			WorkspaceEventInner::Leave(e) => Self::UserLeave { name: e.user.name },
			WorkspaceEventInner::Create(e) => Self::FileTreeUpdated { path: e.path },
			WorkspaceEventInner::Delete(e) => Self::FileTreeUpdated { path: e.path },
			WorkspaceEventInner::Rename(e) => Self::FileTreeUpdated { path: e.after },
		}
	}
}

impl From<&WorkspaceEventInner> for Event {
	fn from(event: &WorkspaceEventInner) -> Self {
		Self::from(event.clone())
	}
}
