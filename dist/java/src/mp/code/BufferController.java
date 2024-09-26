package mp.code;

import mp.code.data.TextChange;
import mp.code.exceptions.ControllerException;

import java.util.Optional;
import java.util.function.Consumer;

/**
 * Allows interaction with a CodeMP buffer, which in simple terms is a document
 * that multiple people can edit concurrently.
 * <p>
 * It is generally safer to avoid storing this directly, see the api notes for {@link Workspace}.
 */
public final class BufferController {
	private final long ptr;

	BufferController(long ptr) {
		this.ptr = ptr;
		Extensions.CLEANER.register(this, () -> free(ptr));
	}

	private static native String get_name(long self);

	/**
	 * Gets the name (path) of the buffer.
	 * @return the path of the buffer
	 */
	public String getName() {
		return get_name(this.ptr);
	}

	private static native String get_content(long self) throws ControllerException;

	/**
	 * Gets the contents of the buffer as a flat string.
	 * This may return incomplete results if called immediately after attaching.
	 * @return the contents fo the buffer as a flat string
	 * @throws ControllerException if the controller was stopped
	 */
	public String getContent() throws ControllerException {
		return get_content(this.ptr);
	}

	private static native TextChange try_recv(long self) throws ControllerException;

	/**
	 * Tries to get a {@link TextChange} from the queue if any were present, and returns
	 * an empty optional otherwise.
	 * @return the first text change in queue, if any are present
	 * @throws ControllerException if the controller was stopped
	 */
	public Optional<TextChange> tryRecv() throws ControllerException {
		return Optional.ofNullable(try_recv(this.ptr));
	}

	private static native TextChange recv(long self) throws ControllerException;

	/**
	 * Blocks until a {@link TextChange} is available and returns it.
	 * @return the text change update that occurred
	 * @throws ControllerException if the controller was stopped
	 */
	public TextChange recv() throws ControllerException {
		return recv(this.ptr);
	}

	private static native void send(long self, TextChange change) throws ControllerException;

	/**
	 * Tries to send a {@link TextChange} update.
	 * @throws ControllerException if the controller was stopped
	 */
	public void send(TextChange change) throws ControllerException {
		send(this.ptr, change);
	}

	private static native void callback(long self, Consumer<BufferController> cb);

	/**
	 * Registers a callback to be invoked whenever a {@link TextChange} occurs.
	 * This will not work unless a Java thread has been dedicated to the event loop.
	 * @see Extensions#drive(boolean)
	 */
	public void callback(Consumer<BufferController> cb) {
		callback(this.ptr, cb);
	}

	private static native void clear_callback(long self);

	/**
	 * Clears the registered callback.
	 * @see #callback(Consumer)
	 */
	public void clearCallback() {
		clear_callback(this.ptr);
	}

	private static native void poll(long self) throws ControllerException;

	/**
	 * Blocks until a {@link TextChange} is available.
	 * @throws ControllerException if the controller was stopped
	 */
	public void poll() throws ControllerException {
		poll(this.ptr);
	}

	private static native void free(long self);

	static {
		NativeUtils.loadLibraryIfNeeded();
	}
}
