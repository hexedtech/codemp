package mp.code.exceptions;

/**
 * An exception returned by the server as a response.
 */
public abstract class ConnectionRemoteException extends Exception {
	protected ConnectionRemoteException(String msg) {
		super(msg);
	}
}
