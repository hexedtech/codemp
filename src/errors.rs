pub type RemoteResult<T> = std::result::Result<T, RemoteError>;

#[derive(Debug, thiserror::Error)]
#[error("server rejected procedure with error code: {0}")]
pub struct RemoteError(#[from] tonic::Status);



pub type ConnectionResult<T> = std::result::Result<T, ConnectionError>;

#[derive(Debug, thiserror::Error)]
pub enum ConnectionError {
	#[error("network error: {0}")]
	Transport(#[from] tonic::transport::Error),

	#[error("server rejected connection attempt: {0}")]
	Remote(#[from] RemoteError),
}

impl From<tonic::Status> for ConnectionError {
	fn from(value: tonic::Status) -> Self {
		Self::Remote(RemoteError(value))
	}
}



pub type ControllerResult<T> = std::result::Result<T, ControllerError>;

#[derive(Debug, thiserror::Error)]
pub enum ControllerError {
	#[error("worker is already stopped")]
	Stopped,

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
