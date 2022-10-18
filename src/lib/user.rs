use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct UserCursor{
	pub buffer: i64,
	pub x: i64,
	pub y: i64
}

impl Display for UserCursor {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "Cursor(buffer:{}, x:{}, y:{})", self.buffer, self.x, self.y)
	}
}


#[derive(Debug, Clone)]
pub struct User {
	pub name: String,
	pub cursor: UserCursor,
}

impl Display for User {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "User(name:{}, cursor:{})", self.name, self.cursor)
	}
}
