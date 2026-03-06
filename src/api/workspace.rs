//! # Workspace
//! A workspace is a working environment containing many buffers, owned by one user.
//! Many users can be invited and join a workspace, accessing its buffer list and being able to
//! attach to its buffers (depending on permissions) to send changes. Workspaces are namespaced to
//! users, meaning two workspaces with the same name can exist, but one user can own only one
//! workspace with a given name.

/// Represents a service workspace
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "py", pyo3::pyclass)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct WorkspaceIdentifier {
	/// Workspace name, cannot change and is unique per owner.
	pub workspace: String,
	/// Workspace owning user
	pub user: String,
}

impl From<codemp_proto::session::WorkspaceIdentifier> for WorkspaceIdentifier {
	fn from(value: codemp_proto::session::WorkspaceIdentifier) -> Self {
		Self {
			workspace: value.workspace,
			user: value.user,
		}
	}
}

impl From<WorkspaceIdentifier> for codemp_proto::session::WorkspaceIdentifier {
	fn from(value: WorkspaceIdentifier) -> Self {
		Self {
			workspace: value.workspace,
			user: value.user,
		}
	}
}

impl std::fmt::Display for WorkspaceIdentifier {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "#{}:{}", self.user, self.workspace)
	}
}
