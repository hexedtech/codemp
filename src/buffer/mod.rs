use std::ops::Range;

pub(crate) mod worker;
pub mod controller;
pub mod factory;


pub struct TextChange {
	pub span: Range<usize>,
	pub content: String,
}
