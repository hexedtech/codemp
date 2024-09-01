
#[deprecated = "use underlying errors to provide more context on what errors could really be thrown"]
#[allow(deprecated)]
pub type Result<T> = std::result::Result<T, Error>;

#[deprecated = "use underlying errors to provide more context on what errors could really be thrown"]
#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("connection error: {0}")]
	Connection(#[from] ConnectionError),

	#[error("procedure error: {0}")]
	Procedure(#[from] ProcedureError),

	#[error("controller error: {0}")]
	Controller(#[from] ControllerError),
}



pub type ProcedureResult<T> = std::result::Result<T, ProcedureError>;

#[derive(Debug, thiserror::Error)]
pub enum ProcedureError {
	#[error("server rejected procedure with error: {0}")]
	Server(#[from] tonic::Status)
}



pub type ConnectionResult<T> = std::result::Result<T, ConnectionError>;

#[derive(Debug, thiserror::Error)]
pub enum ConnectionError {
	#[error("network error: {0}")]
	Transport(#[from] tonic::transport::Error),

	#[error("server rejected connection attempt: {0}")]
	Procedure(#[from] tonic::Status),
}

impl From<ProcedureError> for ConnectionError {
	fn from(value: ProcedureError) -> Self {
		match value {
			ProcedureError::Server(x) => Self::Procedure(x)
		}
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
