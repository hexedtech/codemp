use std::ops::Range;

pub(crate) mod worker;
pub mod controller;
pub mod factory;

pub use factory::OperationFactory;
pub use controller::BufferController as Controller;


/// TODO move in proto
#[derive(Debug)]
pub struct TextChange {
	pub span: Range<usize>,
	pub content: String,
}
