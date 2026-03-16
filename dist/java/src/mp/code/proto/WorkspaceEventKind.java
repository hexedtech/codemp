package mp.code.proto;

/**
 * Represents the kind of workspace event.
 */
public class WorkspaceEventKind {
	/**
	 * Somebody joined a workspace.
	 */
	public static final int USER_JOIN_WORKSPACE = 0;
	/**
	 * Somebody left a workspace.
	 */
	public static final int USER_LEAVE_WORKSPACE = 1;
	/**
	 * A file was created.
	 */
	public static final int FILE_CREATE = 2;
	/**
	 * A file was renamed.
	 */
	public static final int FILE_RENAME = 3;
	/**
	 * A file was deleted.
	 */
	public static final int FILE_DELETE = 4;
	/**
	 * Somebody joined a buffer.
	 */
	public static final int USER_JOIN_BUFFER = 5;
	/**
	 * Somebody left a buffer.
	 */
	public static final int USER_LEAVE_BUFFER = 6;
}
