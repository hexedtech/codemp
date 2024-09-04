//! # User
//! An user is identified by an UUID, which should never change.
//! Each user has an username, which can change but should be unique.

use uuid::Uuid;

/// Represents a service user
#[derive(Debug, Clone)]
pub struct User {
	/// User unique identifier, should never change.
	pub id: Uuid,
	/// User name, can change but should be unique.
	pub name: String,
}

impl From<codemp_proto::common::User> for User {
	fn from(value: codemp_proto::common::User) -> Self {
		Self {
			id: value.id.uuid(),
			name: value.name,
		}
	}
}

impl From<User> for codemp_proto::common::User {
	fn from(value: User) -> Self {
		Self {
			id: value.id.into(),
			name: value.name,
		}
	}
}

impl PartialEq for User {
	fn eq(&self, other: &Self) -> bool {
		self.id.eq(&other.id)
	}
}

impl Eq for User {}

impl PartialOrd for User {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.id.cmp(&other.id))
	}
}

impl Ord for User {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.id.cmp(&other.id)
	}
}
