//! # Workspace
//! A workspace is a working environment containing many buffers, owned by one user.
//! Many users can be invited and join a workspace, accessing its buffer list and being able to
//! attach to its buffers (depending on permissions) to send changes. Workspaces are namespaced to
//! users, meaning two workspaces with the same name can exist, but one user can own only one
//! workspace with a given name.

use uuid::Uuid;

/// Represents a service workspace
#[derive(Debug, Clone)]
#[cfg_attr(feature = "py", pyo3::pyclass)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct WorkspaceInfo {
	/// Workspace unique identifier, should never change.
	pub id: Uuid,
	/// Workspace name, cannot change and is unique per owner.
	pub name: String,
	/// Workspace owning user
	pub owner: super::User,
}

impl From<codemp_proto::common::WorkspaceInfo> for WorkspaceInfo {
	fn from(value: codemp_proto::common::WorkspaceInfo) -> Self {
		Self {
			id: Uuid::from(value.id),
			name: value.name,
			owner: super::User::from(value.owner),
		}
	}
}
