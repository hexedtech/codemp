package mp.code.exceptions;

/**
 * An exception that may occur when a {@link mp.code.BufferController} or
 * a {@link mp.code.CursorController} perform an illegal operation.
 * It may also occur as a result of {@link mp.code.Workspace#event()} and
 * {@link mp.code.Workspace#selectBuffer(long)}.
 */
public abstract class ControllerException extends Exception {
	protected ControllerException(String msg) {
		super(msg);
	}
}
