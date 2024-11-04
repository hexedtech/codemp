#[cfg(all(test, feature = "test-e2e"))]
mod client;

#[cfg(all(test, feature = "test-e2e"))]
mod server;

pub mod fixtures;
use crate::errors::{ConnectionError, RemoteError};

#[derive(Debug)]
pub struct AssertionError(String);

impl AssertionError {
	pub fn new(msg: &str) -> Self {
		Self(msg.to_string())
	}
}

impl std::fmt::Display for AssertionError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.0)
	}
}

impl std::error::Error for AssertionError {}

impl From<ConnectionError> for AssertionError {
	fn from(value: ConnectionError) -> Self {
		match value {
			ConnectionError::Transport(error) => AssertionError::new(&format!(
				"Connection::Transport error during setup of a test: {}",
				error,
			)),
			ConnectionError::Remote(remote_error) => AssertionError::new(&format!(
				"Connection::Remote error during setup of a test: {}",
				remote_error,
			)),
		}
	}
}

impl From<RemoteError> for AssertionError {
	fn from(value: RemoteError) -> Self {
		AssertionError::new(&format!("Remote error during setup of a test: {}", value,))
	}
}

#[macro_export]
macro_rules! assert_or_err {
	($s:expr) => {
		#[allow(clippy::bool_comparison)]
		if !$s {
			return Err($crate::tests::AssertionError::new(&format!(
				"assertion failed at line {}: {}",
				std::line!(),
				stringify!($s)
			))
			.into());
		}
	};
	($s:expr, $msg:literal) => {
		#[allow(clippy::bool_comparison)]
		if !$s {
			return Err($crate::tests::AssertionError::new(&format!(
				"{} (line {})",
				$msg,
				std::line!(),
			))
			.into());
		}
	};
	($s:expr, raw $msg:literal) => {
		#[allow(clippy::bool_comparison)]
		if !$s {
			return Err($crate::tests::AssertionError::new($msg).into());
		}
	};
}

pub use assert_or_err;

#[macro_export]
macro_rules! fixture {
	($fixture:expr => | $($arg:ident),* | $body:expr) => {
		#[allow(unused_parens)]
		$fixture
			.with(|($($arg),*)| {
				$(
					let $arg = $arg.clone();
				)*

				async move {
					$body
				}
			})
			.await;
	};
}

pub use fixture;
