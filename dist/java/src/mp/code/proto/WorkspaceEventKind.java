package mp.code.proto;

import lombok.AccessLevel;
import lombok.RequiredArgsConstructor;

/**
 * Represents the kind of workspace event.
 */
@RequiredArgsConstructor(access = AccessLevel.PRIVATE)
public final class WorkspaceEventKind {
	/**
	 * Event that occurs when a user joins a workspace.
	 */
	public static final int USER_JOIN_WORKSPACE = 0;
	/**
	 * Event that occurs when a user leaves a workspace.
	 */
	public static final int USER_LEAVE_WORKSPACE = 1;
	/**
	 * Event that occurs when a file is created in a workspace.
	 */
	public static final int FILE_CREATE = 2;
	/**
	 * Event that occurs when a file is renamed in a workspace.
	 */
	public static final int FILE_RENAME = 3;
	/**
	 * Event that occurs when a file is deleted in a workspace.
	 */
	public static final int FILE_DELETE = 4;
	/**
	 * Event that occurs when a user joins a buffer.
	 */
	public static final int USER_JOIN_BUFFER = 5;
	/**
	 * Event that occurs when a user leaves a buffer.
	 */
	public static final int USER_LEAVE_BUFFER = 6;
}
