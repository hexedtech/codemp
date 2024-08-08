//! # User
//!
//! data structures for our service users

use uuid::Uuid;

/// represents a service user
///
/// all users are identified uniquely with UUIDs
#[derive(Debug, Clone)]
pub struct User {
	pub id: Uuid,
}

impl From<codemp_proto::common::Identity> for User {
	fn from(value: codemp_proto::common::Identity) -> Self {
		Self {
			id: uuid::Uuid::parse_str(&value.id).expect("invalid uuid"),
		}
	}
}

impl From<User> for codemp_proto::common::Identity {
	fn from(value: User) -> Self {
		Self {
			id: value.id.to_string(),
		}
	}
}
