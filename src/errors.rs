use tracing::warn;

pub trait IgnorableError {
	fn unwrap_or_warn(self, msg: &str);
}

impl<T, E> IgnorableError for Result<T, E>
where E : std::fmt::Display {
	fn unwrap_or_warn(self, msg: &str) {
		match self {
			Ok(_) => {},
			Err(e) => warn!("{}: {}", msg, e),
		}
	}
}
