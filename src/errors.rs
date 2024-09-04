//! ### Errors
//! Contains the crate's error types.

/// An error returned by the server as response to a request.
///
/// This currently wraps an [http code](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status),
/// returned as procedure status.
#[derive(Debug, thiserror::Error)]
#[error("server rejected procedure with error code: {0}")]
pub struct RemoteError(#[from] tonic::Status);

/// Wraps [std::result::Result] with a [RemoteError].
pub type RemoteResult<T> = std::result::Result<T, RemoteError>;

/// An error that may occur when processing requests that require new connections.
#[derive(Debug, thiserror::Error)]
pub enum ConnectionError {
	/// Underlying [`tonic::transport::Error`].
	#[error("transport error: {0}")]
	Transport(#[from] tonic::transport::Error),

	/// Error from the remote server, see [`RemoteError`].
	#[error("server rejected connection attempt: {0}")]
	Remote(#[from] RemoteError),
}

impl From<tonic::Status> for ConnectionError {
	fn from(value: tonic::Status) -> Self {
		Self::Remote(RemoteError(value))
	}
}

/// Wraps [std::result::Result] with a [ConnectionError].
pub type ConnectionResult<T> = std::result::Result<T, ConnectionError>;

/// An error that may occur when an [`crate::api::Controller`] attempts to
/// perform an illegal operation.
#[derive(Debug, thiserror::Error)]
pub enum ControllerError {
	/// Error occurred because the underlying controller worker is already stopped.
	#[error("worker is already stopped")]
	Stopped,

	/// Error occurred because the underlying controller worker stopped before
	/// fulfilling the request, without rejecting it first.
	#[error("worker stopped before completing requested operation")]
	Unfulfilled,
}

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for ControllerError {
	fn from(_: tokio::sync::mpsc::error::SendError<T>) -> Self {
		Self::Stopped
	}
}

impl From<tokio::sync::oneshot::error::RecvError> for ControllerError {
	fn from(_: tokio::sync::oneshot::error::RecvError) -> Self {
		Self::Unfulfilled
	}
}

/// Wraps [std::result::Result] with a [ControllerError].
pub type ControllerResult<T> = std::result::Result<T, ControllerError>;

