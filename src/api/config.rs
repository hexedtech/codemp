//! # Config
//! Data structure defining clients configuration

/// Configuration struct for the `codemp` client.
///
/// `username` and `password` are required fields, everything else is optional.
///
/// `host`, `port` and `tls` affect all connections to all gRPC services; the
/// resulting endpoint is composed like this:
///     http{tls?'s':''}://{host}:{port}
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "js", napi_derive::napi(object))]
#[cfg_attr(
	any(feature = "py", feature = "py-noabi"),
	pyo3::pyclass(get_all, set_all)
)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct Config {
	/// User identifier used to register, possibly your email.
	pub username: String,
	/// User password chosen upon registration.
	pub password: String,
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
			password: password.to_string(),
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
