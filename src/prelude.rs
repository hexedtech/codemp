//! ### Prelude
//! 
//! all-in-one renamed imports with `use codemp::prelude::*`

pub use crate::{
	Error as CodempError,
	Result as CodempResult,
	
	Client as CodempClient,
	api::Controller as CodempController,
	api::OperationFactory as CodempOperationFactory,
	cursor::Controller as CodempCursorController,
	buffer::Controller as CodempBufferController,

	ot::OperationSeq as CodempOperationSeq,
	buffer::TextChange as CodempTextChange,

	proto::CursorPosition as CodempCursorPosition,
	proto::CursorEvent as CodempCursorEvent,
	proto::RowCol as CodempRowCol,

	Instance as CodempInstance,
};

#[cfg(feature = "global")]
pub use crate::instance::global::INSTANCE as CODEMP_INSTANCE;
