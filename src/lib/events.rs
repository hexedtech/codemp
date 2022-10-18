use std::fmt::Display;
use crate::user::User;

#[derive(Debug, Clone)]
pub enum Event {
	UserJoin { user: User },
	UserLeave { name: String },
	BufferNew { path: String },
	BufferDelete { path: String },
}

impl Display for Event {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::UserJoin     { user }  => write!(f, "UserJoin(user:{})",     user),
			Self::UserLeave    { name }  => write!(f, "UserLeave(user:{})",    name),
			Self::BufferNew    { path }  => write!(f, "BufferNew(path:{})",    path),
			Self::BufferDelete { path }  => write!(f, "BufferDelete(path:{})", path),
		}
	}
}

// pub type Event = Box<dyn EventInterface>;
// 
// pub trait EventInterface {
// 	fn class(&self) -> EventClass;
// 	fn unwrap(e: Event) -> Option<Self> where Self: Sized;
// 
// 	fn wrap(self) -> Event {
// 		Box::new(self)
// 	}
// }
// 
// 
// // User joining workspace
// 
// pub struct UserJoinEvent {
// 	user: User,
// }
// 
// impl EventInterface for UserJoinEvent {
// 	fn class(&self) -> EventClass { EventClass::UserJoin }
// 	fn unwrap(e: Event) -> Option<Self> where Self: Sized {
// 		if matches!(e.class(), EventClass::UserJoin) {
// 			return Some(*e);
// 		}
// 		None
// 	}
// }
// 
// 
// // User leaving workspace
// 
// pub struct UserLeaveEvent {
// 	name: String,
// }
// 
// impl EventInterface for UserLeaveEvent {
// 	fn class(&self) -> EventClass { EventClass::UserLeave }
// }
// 
// 
// // Cursor movement
// 
// pub struct CursorEvent {
// 	user: String,
// 	cursor: UserCursor,
// }
// 
// impl EventInterface for CursorEvent {
// 	fn class(&self) -> EventClass { EventClass::Cursor }
// }
// 
// impl CursorEvent {
// 	pub fn new(user:String, cursor: UserCursor) -> Self {
// 		CursorEvent { user, cursor }
// 	}
// }
// 
// 
// // Buffer added
// 
// pub struct BufferNewEvent {
// 	path: String,
// }
// 
// impl EventInterface for BufferNewEvent {
// 	fn class(&self) -> EventClass { EventClass::BufferNew }
// }
// 
// 
// // Buffer deleted
// 
// pub struct BufferDeleteEvent {
// 	path: String,
// }
// 
// impl EventInterface for BufferDeleteEvent {
// 	fn class(&self) -> EventClass { EventClass::BufferDelete }
// }
