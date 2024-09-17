package mp.code.exceptions;

/**
 * An exception that occurred from the underlying tonic layer.
 */
public abstract class ConnectionTransportException extends Exception {

	/**
	 * Creates a new exception with the given message.
	 * @param message the message
	 */
	public ConnectionTransportException(String message) {
		super(message);
	}
}
