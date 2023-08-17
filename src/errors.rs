use std::{error::Error as StdError, fmt::Display};

use tokio::sync::{mpsc, broadcast};
use tonic::{Status, Code};
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

// TODO split this into specific errors for various parts of the library
#[derive(Debug)]
pub enum Error {
	Transport {
		status: Code,
		message: String,
	},
	Channel {
		send: bool
	},
	InvalidState {
		msg: String,
	},

	// TODO filler error, remove later
	Filler {
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

impl From<Status> for Error {
	fn from(status: Status) -> Self {
		Error::Transport { status: status.code(), message: status.message().to_string() }
	}
}

impl From<tonic::transport::Error> for Error {
	fn from(err: tonic::transport::Error) -> Self {
		Error::Transport {
			status: Code::Unknown, message: format!("underlying transport error: {:?}", err)
		}
	}
}

impl<T> From<mpsc::error::SendError<T>> for Error {
	fn from(_value: mpsc::error::SendError<T>) -> Self {
		Error::Channel { send: true }
	}
}

impl From<broadcast::error::RecvError> for Error {
	fn from(_value: broadcast::error::RecvError) -> Self {
		Error::Channel { send: false }
	}
}
