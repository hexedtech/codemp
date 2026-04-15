
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
	UserJoinBuffer = 3,
	UserLeaveBuffer = 4,
	BufferCreate = 5,
	BufferRename = 6,
	BufferDelete = 7,
	BufferAttrsUpdated = 8,
}

return {
	SessionEventKind = SessionEventKind,
	WorkspaceEventKind = WorkspaceEventKind,
}
