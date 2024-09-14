//! # Config
//! Data structure defining clients configuration

/// Configuration struct for `codemp` client
///
/// username and password are required fields, while everything else is optional
///
/// host, port and tls affect all connections to all grpc services
/// resulting endpoint is composed like this:
///     http{tls?'s':''}://{host}:{port}
#[derive(Clone, Debug)]
#[cfg_attr(feature = "js", napi_derive::napi(object))]
#[cfg_attr(feature = "py", pyo3::pyclass(get_all, set_all))]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct Config {
	/// user identifier used to register, possibly your email
	pub username: String,
	/// user password chosen upon registration
	pub password: String,
	/// address of server to connect to, default api.code.mp
	pub host: Option<String>,
	/// port to connect to, default 50053
	pub port: Option<u16>,
	/// enable or disable tls, default true
	pub tls: Option<bool>,
}

impl Config {
	/// construct a new Config object, with given username and password
	pub fn new(username: String, password: String) -> Self {
		Self {
			username,
			password,
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
