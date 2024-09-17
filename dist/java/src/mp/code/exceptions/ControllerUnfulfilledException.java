package mp.code.exceptions;

/**
 * An exception that occurs when the underlying controller stopped before
 * fulfilling the request, without rejecting it first.
 */
public class ControllerUnfulfilledException extends ControllerException {

	/**
	 * Creates a new exception with the given message.
	 * @param message the message
	 */
	public ControllerUnfulfilledException(String message) {
		super(message);
	}
}
