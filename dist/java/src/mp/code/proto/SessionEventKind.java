package mp.code.proto;

import lombok.AccessLevel;
import lombok.RequiredArgsConstructor;

/**
 * Represents the kind of session event.
 */
@RequiredArgsConstructor(access = AccessLevel.PRIVATE)
public class SessionEventKind {
	/**
	 * Event that occurs when you get invited to a workspace.
	 */
	public static final int INVITATION_EVENT = 0;

	/**
	 * Event that occurs when a user quits a workspace.
	 */
	public static final int QUIT_EVENT = 1;

	/**
	 * Event that occurs when a user accepts an invitation to a workspace you are in.
	 */
	public static final int ACCEPT_EVENT = 2;

	/**
	 * Event that occurs when a user reject an invite.
	 */
	public static final int REJECT_EVENT = 3;
}
