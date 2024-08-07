package mp.code.exceptions;

/**
 * A generic class for all our exceptions coming through the JNI from the library.
 */
public abstract class CodeMPException extends Exception {
	protected CodeMPException(String msg) {
		super(msg);
	}
}
