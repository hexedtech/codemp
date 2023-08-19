use std::ops::Range;

pub(crate) mod worker;
pub mod controller;
pub mod factory;


#[derive(Debug)]
pub struct TextChange {
	pub span: Range<usize>,
	pub content: String,
}
