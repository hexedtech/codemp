package mp.code;

import java.io.IOException;

public class Extensions {
	private static boolean loaded = false;
	static synchronized void loadLibraryIfNotPresent() {
		if(loaded) return;
		try {
			String filename = System.getProperty("os.name").startsWith("Windows")
				? "/natives/codemp.dll"
				: "/natives/libcodemp.so";
			cz.adamh.utils.NativeUtils.loadLibraryFromJar(filename);
			loaded = true;
		} catch(IOException e) {
			throw new RuntimeException(e);
		}
	}

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

	/**
	 * Configures the tracing subscriber for the native logs.
	 * @param path where to output this, null to use stdout
	 * @param debug whether to run it in debug mode
	 */
	public static native void setupTracing(String path, boolean debug);

	static {
		Extensions.loadLibraryIfNotPresent();
	}
}
