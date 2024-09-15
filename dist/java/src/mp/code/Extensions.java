package mp.code;

public class Extensions {
	/**
	 * Hashes the given {@link String} using CodeMP's hashing algorithm (xxh3).
	 * @param input the string to hash
	 * @return the hash
	 */
	public static native long hash(String input);

	/**
	 * Drive the underlying library's asynchronous event loop.
	 * @param block true if it should use the current thread, false if it should
	 *              spawn a separate one
	 */
	public static native void drive(boolean block);
}
