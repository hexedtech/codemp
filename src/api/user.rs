//! # User
//! An user is identified by an UUID, which should never change.
//! Each user has an username, which can change but should be unique.

/// Represents a service user
#[derive(Debug, Clone)]
#[cfg_attr(feature = "java", jni_toolbox::jclass(class = "mp/code/data/UserInfo"))]
#[cfg_attr(feature = "py", pyo3::pyclass(from_py_object))]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct UserInfo {
	/// User name, unique and immutable
	pub name: String,
	/// User display name, can change and be duplicated
	#[cfg_attr(feature = "serialize", serde(skip_serializing_if = "Option::is_none"))]
	pub display_name: Option<String>,
	/// User description ("bio"), may contain contacts
	#[cfg_attr(feature = "serialize", serde(skip_serializing_if = "Option::is_none"))]
	pub description: Option<String>,
	/// User avatar: a small image some editors can display
	#[cfg_attr(feature = "serialize", serde(skip_serializing_if = "Option::is_none"))]
	pub avatar: Option<Vec<u8>>,
}

impl UserInfo {
	/// Creates a default [UserInfo] for the given username
	/// with all the optional fields set to [None].
	pub fn default_for(username: String) -> Self {
		Self {
			name: username,
			display_name: None,
			description: None,
			avatar: None,
		}
	}
}

impl From<codemp_proto::common::UserInfo> for UserInfo {
	fn from(value: codemp_proto::common::UserInfo) -> Self {
		Self {
			name: value.name,
			display_name: value.display_name,
			description: value.description,
			avatar: value.avatar,
		}
	}
}

impl From<UserInfo> for codemp_proto::common::UserInfo {
	fn from(value: UserInfo) -> Self {
		Self {
			name: value.name,
			display_name: value.display_name,
			description: value.description,
			avatar: value.avatar,
		}
	}
}

impl PartialEq for UserInfo {
	fn eq(&self, other: &Self) -> bool {
		self.name.eq(&other.name)
	}
}

impl Eq for UserInfo {}

impl PartialOrd for UserInfo {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for UserInfo {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.name.cmp(&other.name)
	}
}
