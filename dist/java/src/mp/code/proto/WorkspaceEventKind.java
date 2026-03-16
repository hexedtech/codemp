package mp.code.proto;

/**
 * Represents the kind of workspace event.
 */
public enum WorkspaceEventKind {
	/**
	 * Somebody joined a workspace.
	 */
	USER_JOIN_WORKSPACE,
	/**
	 * Somebody left a workspace.
	 */
	USER_LEAVE_WORKSPACE,
	/**
	 * A file was created.
	 */
	FILE_CREATE,
	/**
	 * A file was renamed.
	 */
	FILE_RENAME,
	/**
	 * A file was deleted.
	 */
	FILE_DELETE,
	/**
	 * Somebody joined a buffer.
	 */
	USER_JOIN_BUFFER,
	/**
	 * Somebody left a buffer.
	 */
	USER_LEAVE_BUFFER
}
