package mp.code.exceptions;

/**
 * An exception that occurred from the underlying tonic layer.
 */
public abstract class ConnectionTransportException extends Exception {
	protected ConnectionTransportException(String msg) {
		super(msg);
	}
}
