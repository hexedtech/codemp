package mp.code.exceptions;

/**
 * An exception that may occur when a {@link mp.code.BufferController} or
 * a {@link mp.code.CursorController} or {@link mp.code.Workspace} (in the
 * receiver part) perform an illegal operation.
 */
public abstract class ControllerException extends Exception {

	/**
	 * Creates a new exception with the given message.
	 * @param message the message
	 */
	protected ControllerException(String message) {
		super(message);
	}
}
