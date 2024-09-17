package mp.code.exceptions;

/**
 * An exception that occurs when attempting to send an operation when
 * the worker has already stopped.
 */
public class ControllerStoppedException extends ControllerException {

	/**
	 * Creates a new exception with the given message.
	 * @param message the message
	 */
	public ControllerStoppedException(String message) {
		super(message);
	}
}
