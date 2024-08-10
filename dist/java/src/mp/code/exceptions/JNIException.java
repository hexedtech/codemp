package mp.code.exceptions;

/**
 * Thrown when an error happened in some jni-rs method that would've otherwise crashed
 * the program. This way, the eventual crash can happen on the Java side.
 * Only catch this if you are aware of the implications.
 */
public class JNIException extends RuntimeException {
	public JNIException(String message) {
		super(message);
	}
}
