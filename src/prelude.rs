//! ### Prelude
//! 
//! all-in-one renamed imports with `use codemp::prelude::*`

pub use crate::{
	Error as CodempError,
	Result as CodempResult,
};

#[cfg(feature = "ot")]
pub use	crate::ot::OperationSeq as CodempOperationSeq;

#[cfg(feature = "api")]
pub use crate::{
	api::Controller as CodempController,
	api::OperationFactory as CodempOperationFactory,
};
	
#[cfg(feature = "client")]
pub use crate::{
	client::Client as CodempClient,
	cursor::Controller as CodempCursorController,
	buffer::Controller as CodempBufferController,
	buffer::TextChange as CodempTextChange,
	Instance as CodempInstance,
};

#[cfg(feature = "proto")]
pub use crate::{
	proto::CursorPosition as CodempCursorPosition,
	proto::CursorEvent as CodempCursorEvent,
	proto::RowCol as CodempRowCol,
};

#[cfg(feature = "global")]
pub use crate::instance::global::INSTANCE as CODEMP_INSTANCE;
