package mp.code;

public class Utils {
	/**
	 * Hashes the given {@link String} using CodeMP's hashing algorithm (xxh3).
	 * @param input the string to hash
	 * @return the hash
	 */
	public static native long hash(String input);
}
