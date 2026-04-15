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
	public static final int USER_JOIN_WORKSPACE = 1;
	/**
	 * Event that occurs when a user leaves a workspace.
	 */
	public static final int USER_LEAVE_WORKSPACE = 2;
	/**
	 * Event that occurs when a user joins a buffer.
	 */
	public static final int USER_JOIN_BUFFER = 3;
	/**
	 * Event that occurs when a user leaves a buffer.
	 */
	public static final int USER_LEAVE_BUFFER = 4;
	/**
	 * Event that occurs when a buffer is created in a workspace.
	 */
	public static final int BUFFER_CREATE = 5;
	/**
	 * Event that occurs when a buffer is renamed in a workspace.
	 */
	public static final int BUFFER_RENAME = 6;
	/**
	 * Event that occurs when a buffer is deleted in a workspace.
	 */
	public static final int BUFFER_DELETE = 7;
	/**
	 * Event that occurs when a buffer has one of its attributes changed.
	 */
	public static final int BUFFER_ATTRS_UPDATED = 8;
}
