use codemp_proto::workspace::workspace_event::Event as WorkspaceEventInner;

pub enum Event {
	FileTreeUpdated,
	UserJoin(String),
	UserLeave(String),
}

impl From<&WorkspaceEventInner> for Event {
	fn from(event: &WorkspaceEventInner) -> Self {
		match event {
			WorkspaceEventInner::Join(e) => Self::UserJoin(e.user.id.clone()),
			WorkspaceEventInner::Leave(e) => Self::UserLeave(e.user.id.clone()),
			WorkspaceEventInner::Create(_)
			| WorkspaceEventInner::Rename(_)
			| WorkspaceEventInner::Delete(_) => Self::FileTreeUpdated,
		}
	}
}
