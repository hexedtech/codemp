package mp.code;

import java.io.IOException;

/**
 * A class holding utility functions, as well as functions which are specific
 * to this language's glue and don't necessarily have a counterpart in the
 * broader CodeMP API.
 */
public final class Extensions {
	/**
	 * Hashes the given {@link String} using CodeMP's hashing algorithm (xxh3).
	 * @param input the string to hash
	 * @return the hash
	 */
	public static native long hash(String input);

	/**
	 * Drive the underlying library's asynchronous event loop. In other words, tells
	 * it what thread to use. You usually want to call this during initialisation.
	 * <p>
	 * Passing false will have the native library manage threads, but it may fail to
	 * work with some more advanced features.
	 * <p>
	 * You may alternatively call this with true, in a separate and dedicated Java thread;
	 * it will remain active in the background and act as event loop. Assign it like this:
	 * <p><code>new Thread(() -> Extensions.drive(true)).start();</code></p>
	 * @param block true if it should use the current thread
	 */
	public static native void drive(boolean block);

	/**
	 * Configures the tracing subscriber for the native logs.
	 * Do not call this unless you want to see the native library's logs.
	 * @param path where to output this, null to use stdout
	 * @param debug whether to run it in debug mode
	 */
	public static native void setupTracing(String path, boolean debug);

	static {
		NativeUtils.loadLibraryIfNeeded();
	}
}
