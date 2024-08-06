package mp.code.exceptions;

/**
 * A generic class for all our exceptions coming through the JNI from the library.
 */
public abstract class CodeMPLibException extends Exception {
	protected CodeMPLibException(String msg) {
		super(msg);
	}
}
