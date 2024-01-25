//! ### Errors
//! 
//! library error helpers and types

use std::{result::Result as StdResult, error::Error as StdError, fmt::Display};

use tracing::warn;

/// an error which can be ignored with just a warning entry
pub trait IgnorableError {
	fn unwrap_or_warn(self, msg: &str);
}

impl<T, E> IgnorableError for StdResult<T, E>
where E : std::fmt::Debug {
	fn unwrap_or_warn(self, msg: &str) {
		match self {
			Ok(_) => {},
			Err(e) => warn!("{}: {:?}", msg, e),
		}
	}
}


/// an error which can be ignored with just a warning entry and returning the default value
pub trait IgnorableDefaultableError<T> {
	fn unwrap_or_warn_default(self, msg: &str) -> T;
}

impl<T, E> IgnorableDefaultableError<T> for StdResult<T, E>
where E : std::fmt::Display, T: Default {
	fn unwrap_or_warn_default(self, msg: &str) -> T {
		match self {
			Ok(x) => x,
			Err(e) => {
				warn!("{}: {}", msg, e);
				T::default()
			},
		}
	}
}

/// result type for codemp errors
pub type Result<T> = StdResult<T, Error>;

// TODO split this into specific errors for various parts of the library
/// codemp error type for library issues
#[derive(Debug)]
pub enum Error {
	/// errors caused by tonic http layer
	Transport {
		status: String,
		message: String,
	},
	/// errors caused by async channels
	Channel {
		send: bool
	},
	/// errors caused by wrong usage of library objects
	InvalidState {
		msg: String,
	},

	/// errors caused by wrong interlocking, safe to retry
	Deadlocked,

	/// if you see these errors someone is being lazy (:
	Filler { // TODO filler error, remove later
		message: String,
	},
}

impl StdError for Error {}

impl Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Transport { status, message } => write!(f, "Transport error: ({}) {}", status, message),
			Self::Channel { send } => write!(f, "Channel error (send:{})", send),
			_ => write!(f, "Unknown error"),
		}
	}
}

#[cfg(feature = "client")]
impl From<tonic::Status> for Error {
	fn from(status: tonic::Status) -> Self {
		Error::Transport {
			status: status.code().to_string(),
			message: status.message().to_string()
		}
	}
}

#[cfg(feature = "client")]
impl From<tonic::transport::Error> for Error {
	fn from(err: tonic::transport::Error) -> Self {
		Error::Transport {
			status: tonic::Code::Unknown.to_string(),
			message: format!("underlying transport error: {:?}", err)
		}
	}
}

#[cfg(feature = "client")]
impl<T> From<tokio::sync::mpsc::error::SendError<T>> for Error {
	fn from(_value: tokio::sync::mpsc::error::SendError<T>) -> Self {
		Error::Channel { send: true }
	}
}

#[cfg(feature = "client")]
impl<T> From<tokio::sync::watch::error::SendError<T>> for Error {
	fn from(_value: tokio::sync::watch::error::SendError<T>) -> Self {
		Error::Channel { send: true }
	}
}

#[cfg(feature = "client")]
impl From<tokio::sync::broadcast::error::RecvError> for Error {
	fn from(_value: tokio::sync::broadcast::error::RecvError) -> Self {
		Error::Channel { send: false }
	}
}

#[cfg(feature = "client")]
impl From<tokio::sync::watch::error::RecvError> for Error {
	fn from(_value: tokio::sync::watch::error::RecvError) -> Self {
		Error::Channel { send: false }
	}
}
