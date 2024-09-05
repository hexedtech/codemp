package mp.code.exceptions;

/**
 * An exception that occurs when the underlying controller stopped before
 * fulfilling the request, without rejecting it first.
 */
public class ControllerUnfulfilledException extends ControllerException {
	protected ControllerUnfulfilledException(String msg) {
		super(msg);
	}
}
