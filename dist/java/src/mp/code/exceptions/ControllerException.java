package mp.code.exceptions;

/**
 * An exception that may occur when a {@link mp.code.BufferController} or
 * a {@link mp.code.CursorController} perform an illegal operation.
 * It may also occur as a result of {@link mp.code.Workspace#event()}.
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
