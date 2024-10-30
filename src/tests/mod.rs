#[cfg(all(test, feature = "test-e2e"))]
mod client;

#[cfg(all(test, feature = "test-e2e"))]
mod server;

pub mod fixtures;

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

#[macro_export]
macro_rules! assert_or_err {
	($s:expr) => {
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
		if !$s {
			return Err($crate::tests::AssertionError::new($msg).into());
		}
	};
}

pub use assert_or_err;
