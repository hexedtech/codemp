//! # Event
//! Real time notification of changes in a workspace, to either users or buffers.
#![allow(non_upper_case_globals, non_camel_case_types)] // pyo3 fix your shit

use codemp_proto::workspace::workspace_event::Event as WorkspaceEventInner;

/// Event in a [crate::Workspace].
#[derive(Debug, Clone)]
#[cfg_attr(feature = "py", pyo3::pyclass(from_py_object))]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serialize", serde(tag = "type"))]
pub enum Event {
	/// Fired when the file tree changes (buffer created, deleted or renamed).
	FileTreeUpdated {
		/// The modifier buffer's path.
		path: String
	},
	/// Fired when an user joins the current workspace.
	UserJoin {
		/// The name of the joining user.
		name: String
	},
	/// Fired when an user leaves the current workspace.
	UserLeave {
		/// The name of the leaving user.
		name: String
	},
	/// Fired when an user joins a buffer.
	UserJoinBuffer {
		/// The name of the joining user.
		name: String,
		/// The name of the buffer the user is joining.
		buffer: String
	},
	/// Fired when an user leaves a buffer.
	UserLeaveBuffer {
		/// The name of the leaving user.
		name: String,
		/// The name of the buffer the user is leaving.
		buffer: String
	},
}

impl From<WorkspaceEventInner> for Event {
	fn from(event: WorkspaceEventInner) -> Self {
		match event {
			WorkspaceEventInner::WorkspaceJoin(e) => Self::UserJoin { name: e.user },
			WorkspaceEventInner::WorkspaceLeave(e) => Self::UserLeave { name: e.user },
			WorkspaceEventInner::Create(e) => Self::FileTreeUpdated { path: e.path },
			WorkspaceEventInner::Delete(e) => Self::FileTreeUpdated { path: e.path },
			WorkspaceEventInner::Rename(e) => Self::FileTreeUpdated { path: e.after },
			WorkspaceEventInner::BufferJoin(e) => Self::UserJoinBuffer {
				name: e.user,
				buffer: e.buffer,
			},
			WorkspaceEventInner::BufferLeave(e) => Self::UserLeaveBuffer {
				name: e.user,
				buffer: e.buffer,
			},
		}
	}
}

impl From<&WorkspaceEventInner> for Event {
	fn from(event: &WorkspaceEventInner) -> Self {
		Self::from(event.clone())
	}
}
