package mp.code;

import mp.code.data.Cursor;
import mp.code.data.Selection;
import mp.code.exceptions.ControllerException;

import java.util.Optional;
import java.util.function.Consumer;

/**
 * Allows interaction with the CodeMP cursor position tracking system.
 * <p>
 * It is generally safer to avoid storing this directly, see the api notes for {@link Workspace}.
 */
public final class CursorController {
	private final long ptr;

	CursorController(long ptr) {
		this.ptr = ptr;
		Extensions.CLEANER.register(this, () -> free(ptr));
	}

	private static native Cursor try_recv(long self) throws ControllerException;

	/**
	 * Tries to get a {@link Cursor} update from the queue if any were present, and returns
	 * an empty optional otherwise.
	 * @return the first cursor event in queue, if any are present
	 * @throws ControllerException if the controller was stopped
	 */
	public Optional<Cursor> tryRecv() throws ControllerException {
		return Optional.ofNullable(try_recv(this.ptr));
	}

	private static native Cursor recv(long self) throws ControllerException;

	/**
	 * Blocks until a {@link Cursor} update is available and returns it.
	 * @return the cursor update that occurred
	 * @throws ControllerException if the controller was stopped
	 */
	public Cursor recv() throws ControllerException {
		return recv(this.ptr);
	}

	private static native void send(long self, Selection cursor) throws ControllerException;

	/**
	 * Tries to send a {@link Selection} update.
	 * @throws ControllerException if the controller was stopped
	 */
	public void send(Selection cursor) throws ControllerException {
		send(this.ptr, cursor);
	}

	private static native void callback(long self, Consumer<CursorController> cb);

	/**
	 * Registers a callback to be invoked whenever a {@link Cursor} update occurs.
	 * This will not work unless a Java thread has been dedicated to the event loop.
	 * @see Extensions#drive(boolean)
	 */
	public void callback(Consumer<CursorController> cb) {
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
	 * Blocks until a {@link Cursor} update is available.
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
