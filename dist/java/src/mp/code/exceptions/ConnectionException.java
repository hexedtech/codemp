package mp.code.exceptions;

/**
 * An exception that may occur when processing network requests.
 */
public abstract class ConnectionException extends Exception {

	/**
	 * Creates a new exception with the given message.
	 * @param message the message
	 */
	protected ConnectionException(String message) {
		super(message);
	}
}
