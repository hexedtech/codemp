//! # Config
//! Data structure defining clients configuration


/// Configuration struct for `codemp` client
#[derive(Debug, Clone)]
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
			"{}{}:{}",
			if self.tls() { "https://" } else { "http" },
			self.host(),
			self.port()
		)
	}
}
