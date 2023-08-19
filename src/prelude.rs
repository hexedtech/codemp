pub use crate::client::Client as CodempClient;
pub use crate::errors::Error as CodempError;

pub use crate::Controller as CodempController;
pub use crate::cursor::controller::CursorController as CodempCursorController;
pub use crate::buffer::controller::BufferController as CodempBufferController;

pub use crate::buffer::factory::OperationFactory as CodempOperationFactory;
pub use operational_transform::OperationSeq as CodempOperationSeq;
pub use crate::buffer::TextChange as CodempTextChange;

pub use crate::proto::{
	CursorPosition as CodempCursorPosition,
	CursorEvent as CodempCursorEvent,
	RowCol as CodempRowCol,
};

#[cfg(feature = "sync")]
pub use crate::instance::sync::Instance as CodempInstance;

#[cfg(not(feature = "sync"))]
pub use crate::instance::a_sync::Instance as CodempInstance;

#[cfg(feature = "global")]
pub use crate::instance::global::INSTANCE as CODEMP_INSTANCE;
