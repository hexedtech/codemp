package mp.code.exceptions;

/**
 * An exception that may occur when processing network requests.
 */
public abstract class ConnectionException extends Exception {
	protected ConnectionException(String msg) {
		super(msg);
	}
}
