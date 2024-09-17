package mp.code.data;

import mp.code.Workspace;

/**
 * The result of a {@link Workspace#detachFromBuffer(String)} operation.
 */
public enum DetachResult {
	/** The user was not attached to this buffer. */
	NOT_ATTACHED,
	/** The user detached from the buffer and stopped it. */
	DETACHING,
	/** The user was attached, but the buffer was already stopped. */
	ALREADY_DETACHED
}
