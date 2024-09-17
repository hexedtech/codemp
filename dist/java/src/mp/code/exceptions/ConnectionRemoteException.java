package mp.code.exceptions;

/**
 * An exception returned by the server as a response.
 */
public abstract class ConnectionRemoteException extends ConnectionException {

	/**
	 * Creates a new exception with the given message.
	 * @param message the message
	 */
	public ConnectionRemoteException(String message) {
		super(message);
	}
}
