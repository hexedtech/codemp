package mp.code;

import mp.code.proto.CursorUpdate;
import mp.code.proto.CursorEvent;
import mp.code.proto.CursorPosition;
import mp.code.proto.WorkspaceIdentifier;
import mp.code.exceptions.ControllerException;

import java.util.function.Consumer;

/**
 * Allows interaction with the CodeMP cursor position tracking system.
 * <p>
 *  It is generally safer to avoid storing this directly, see the api notes for {@link Workspace}.
 * </p>
 */
public final class CursorController {
	private final long ptr;

	CursorController(long ptr) {
		this.ptr = ptr;
		Extensions.CLEANER.register(this, () -> free(ptr));
	}

	private static native WorkspaceIdentifier workspace_id(long self);

	/**
	 * Gets the identifier for the one that contains this cursor.
	 * @return a {@link WorkspaceIdentifier} for the owner
	 */
	public WorkspaceIdentifier workspaceId() {
		return workspace_id(this.ptr);
	}

	private static native CursorEvent try_recv(long self) throws ControllerException;

	/**
	 * Tries to get a {@link CursorEvent} from the queue if any were present, null otherwise.
	 * @return the first cursor event in queue, if any are present
	 * @throws ControllerException if the controller was stopped
	 */
	public CursorEvent tryRecv() throws ControllerException {
		return try_recv(this.ptr);
	}

	private static native CursorUpdate recv(long self) throws ControllerException;

	/**
	 * Blocks until a {@link CursorEvent} is available and returns it.
	 * @return the cursor event that occurred
	 * @throws ControllerException if the controller was stopped
	 */
	public CursorUpdate recv() throws ControllerException {
		return recv(this.ptr);
	}

	private static native void send(long self, CursorUpdate selection) throws ControllerException;

	/**
	 * Tries to send a {@link CursorPosition} update.
	 * @param update the update to send
	 * @throws ControllerException if the controller was stopped
	 */
	public void send(CursorUpdate update) throws ControllerException {
		send(this.ptr, update);
	}

	private static native void callback(long self, Consumer<CursorController> cb);

	/**
	 * Registers a callback to be invoked whenever a {@link CursorUpdate} update occurs.
	 * This will not work unless a Java thread has been dedicated to the event loop.
	 * @param cb a {@link Consumer} that receives the controller when the change occurs;
	 *           you should probably spawn a new thread in here, to avoid deadlocking
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
	 * Blocks until a {@link CursorUpdate} update is available.
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
