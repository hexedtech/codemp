
---all enum types for session events
local SessionEventKind = {
	InvitationEvent = 1,
	QuitEvent = 2,
	AcceptEvent = 3,
	RejectEvent = 4,
}

---all enum types for workspace events
local WorkspaceEventKind = {
	UserJoinWorkspace = 1,
	UserLeaveWorkspace = 2,
	FileCreate = 3,
	FileRename = 4,
	FileDelete = 5,
	UserJoinBuffer = 6,
	UserLeaveBuffer = 7,
	FileAttrsUpdated = 8,
}

return {
	SessionEventKind = SessionEventKind,
	WorkspaceEventKind = WorkspaceEventKind,
}
