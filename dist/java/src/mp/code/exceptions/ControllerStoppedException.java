package mp.code.exceptions;

/**
 * An exception that occurs when attempting to send an operation when
 * the worker has already stopped.
 */
public class ControllerStoppedException extends ControllerException {
	protected ControllerStoppedException(String msg) {
		super(msg);
	}
}
