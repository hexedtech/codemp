//! # Config
//! Data structure defining clients configuration

use std::fmt::{Debug, Display};

/// Configuration struct for the `codemp` client.
///
/// `username` and `password` are required fields, everything else is optional.
///
/// `host`, `port` and `tls` affect all connections to all gRPC services; the
/// resulting endpoint is composed like this:
///     http{tls?'s':''}://{host}:{port}
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "js", napi_derive::napi(object))]
#[cfg_attr(feature = "py", pyo3::pyclass(get_all, set_all))]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct Config {
	/// User identifier used to register, possibly your email.
	pub username: String,
	/// User password chosen upon registration.
	pub password: Password, // must not leak this!
	/// Address of server to connect to, default api.code.mp.
	pub host: Option<String>,
	/// Port to connect to, default 50053.
	pub port: Option<u16>,
	/// Enable or disable tls, default true.
	pub tls: Option<bool>,
}

impl Config {
	/// Construct a new Config object, with given username and password.
	pub fn new(username: impl ToString, password: impl ToString) -> Self {
		Self {
			username: username.to_string(),
			password: password.to_string().into(),
			host: None,
			port: None,
			tls: None,
		}
	}

	#[inline]
	pub(crate) fn host(&self) -> &str {
		self.host.as_deref().unwrap_or("api.code.mp")
	}

	#[inline]
	pub(crate) fn port(&self) -> u16 {
		self.port.unwrap_or(50053)
	}

	#[inline]
	pub(crate) fn tls(&self) -> bool {
		self.tls.unwrap_or(true)
	}

	pub(crate) fn endpoint(&self) -> String {
		format!(
			"{}://{}:{}",
			if self.tls() { "https" } else { "http" },
			self.host(),
			self.port()
		)
	}
}

#[derive(Clone, Default)]
#[repr(transparent)]
#[cfg_attr(
	feature = "serialize",
	derive(serde::Serialize, serde::Deserialize),
	serde(transparent)
)]
#[cfg_attr(feature = "py", derive(pyo3::FromPyObject, pyo3::IntoPyObject))]
#[cfg_attr(feature = "js", napi(transparent))]
pub struct Password(String);

impl From<String> for Password {
	fn from(value: String) -> Self {
		Password(value)
	}
}

impl From<&str> for Password {
	fn from(value: &str) -> Self {
		Password(value.to_string())
	}
}

impl From<Password> for String {
	fn from(value: Password) -> Self {
		value.0
	}
}

impl Display for Password {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "********")
	}
}

impl Debug for Password {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "********")
	}
}
