package mp.code;

import java.util.function.Consumer;

import mp.code.proto.UserInfo;
import mp.code.exceptions.ConnectionException;
import mp.code.exceptions.ConnectionRemoteException;
import mp.code.exceptions.ControllerException;
import mp.code.proto.WorkspaceEvent;
import mp.code.proto.WorkspaceIdentifier;

/**
 * Represents a CodeMP workspace, which broadly speaking is a collection
 * of buffers across which edits and cursor movements are tracked.
 * Generally, it is safer to avoid storing this directly. Instead,
 * users should let the native library manage as much as possible for
 * them. They should store the workspace ID and retrieve the object
 * whenever needed with {@link Client#getWorkspace(String, String)}.
 */
public final class Workspace {
	private final long ptr;

	Workspace(long ptr) {
		this.ptr = ptr;
		Extensions.CLEANER.register(this, () -> free(ptr));
	}

	private static native WorkspaceIdentifier id(long self);

	/**
	 * Gets the unique identifier of the current workspace.
	 * @return the {@link WorkspaceIdentifier} for this workspace
	 */
	public WorkspaceIdentifier id() {
		return id(this.ptr);
	}

	private static native CursorController cursor(long self);

	/**
	 * Gets the {@link CursorController} for the current workspace.
	 * @return the {@link CursorController}
	 */
	public CursorController cursor() {
		return cursor(this.ptr);
	}

	private static native BufferController get_buffer(long self, String path);

	/**
	 * Looks for a {@link BufferController} with the given path within the
	 * current workspace and returns it if it exists.
	 * @param path the current path
	 * @return the {@link BufferController} with the given path, if it exists, null otherwise
	 */
	public BufferController getBuffer(String path) {
		return get_buffer(this.ptr, path);
	}

	private static native String[] search_buffers(long self, String filter);

	/**
	 * Searches for buffers matching the filter in this workspace.
	 * @param filter the filter to apply (may be null)
	 * @return an array containing file tree as flat paths
	 */
	public String[] searchBuffers(String filter) {
		return search_buffers(this.ptr, filter);
	}

	private static native String[] active_buffers(long self);

	/**
	 * Returns the currently active buffers (the ones the user is currently
	 * attached to).
	 * @return an array containing the paths of the active buffers
	 */
	public String[] activeBuffers() {
		return active_buffers(this.ptr);
	}

	private static native UserInfo[] user_list(long self);

	/**
	 * Returns the users currently in the workspace.
	 * @return an array containing the users in the workspace
	 */
	public UserInfo[] userList() {
		return user_list(this.ptr);
	}

	private static native void create_buffer(long self, String path, boolean ephemeral) throws ConnectionRemoteException;

	/**
	 * Creates a buffer with the given path.
	 * @param path the new buffer's path
	 * @param ephemeral whether the buffer should be ephemeral
	 * @throws ConnectionRemoteException if an error occurs in communicating with the server
	 */
	public void createBuffer(String path, boolean ephemeral) throws ConnectionRemoteException {
		create_buffer(this.ptr, path, ephemeral);
	}

	private static native void pin_buffer(long self, String path) throws ConnectionRemoteException;

	/**
	 * Pins an ephemeral buffer, making it non-ephemeral.
	 * @param path the buffer's path
	 * @throws ConnectionRemoteException if an error occurs in communicating with the server
	 */
	public void pinBuffer(String path) throws ConnectionRemoteException {
		pin_buffer(this.ptr, path);
	}

	private static native void un_pin_buffer(long self, String path) throws ConnectionRemoteException;

	/**
	 * Unpins a buffer, making it ephemeral.
	 * @param path the buffer's path
	 * @throws ConnectionRemoteException if an error occurs in communicating with the server
	 */
	public void unpinBuffer(String path) throws ConnectionRemoteException {
		un_pin_buffer(this.ptr, path);
	}

	private static native BufferController attach_buffer(long self, String path) throws ConnectionException;

	/**
	 * Attaches to an existing buffer with the given path, if present.
	 * @param path the path of the buffer to attach to
	 * @return the {@link BufferController} associated with that path
	 * @throws ConnectionException if an error occurs in communicating with the server, or if the buffer did not exist
	 */
	public BufferController attachBuffer(String path) throws ConnectionException {
		return attach_buffer(ptr, path);
	}

	private static native boolean detach_buffer(long self, String path);

	/**
	 * Detaches from a given buffer.
	 * @param path the path of the buffer to detach from
	 * @return a boolean, true only if there are still dangling references preventing controller from stopping
	 */
	public boolean detachBuffer(String path) {
		return detach_buffer(this.ptr, path);
	}

	private static native String[] fetch_buffers(long self) throws ConnectionRemoteException;

	/**
	 * Updates and fetches the local list of buffers.
	 * @return the updated list
	 * @throws ConnectionRemoteException if an error occurs in communicating with the server
	 */
	public String[] fetchBuffers() throws ConnectionRemoteException {
		return fetch_buffers(this.ptr);
	}

	private static native void fetch_users(long self) throws ConnectionRemoteException;

	/**
	 * Updates the local list of users.
	 * @throws ConnectionRemoteException if an error occurs in communicating with the server
	 */
	public void fetchUsers() throws ConnectionRemoteException {
		fetch_users(this.ptr);
	}

	private static native void fetch_buffer_users(long self, String path) throws ConnectionRemoteException;

	/**
	 * Updates the local list of users attached to a certain buffer.
	 * The user must be attached to the buffer to perform this operation.
	 * @param path the path of the buffer to search
	 * @throws ConnectionRemoteException if an error occurs in communicating with the server, or the user wasn't attached
	 */
	public void fetchBufferUsers(String path) throws ConnectionRemoteException {
		fetch_buffer_users(this.ptr, path);
	}

	private static native UserInfo[] buffer_user_list(long self, String path);

	/**
	 * Gets the local list of users attached to a certain buffer.
	 * The user must be attached to the buffer to perform this operation.
	 * You can force-update the list with {@link #fetchBufferUsers(String)}.
	 * @param path the path of the buffer to search
	 * @return the local list of users attached to the given buffer
	 */
	public UserInfo[] bufferUserList(String path) {
		return buffer_user_list(this.ptr, path);
	}

	private static native void delete_buffer(long self, String path) throws ConnectionRemoteException;

	/**
	 * Deletes the buffer with the given path.
	 * @param path the path of the buffer to delete
	 * @throws ConnectionRemoteException if an error occurs in communicating with the server
	 */
	public void deleteBuffer(String path) throws ConnectionRemoteException {
		delete_buffer(this.ptr, path);
	}

	private static native WorkspaceEvent try_recv(long self) throws ControllerException;

	/**
	 * Tries to get a {@link WorkspaceEvent} from the queue if any were present, null otherwise
	 * @return the first workspace WorkspaceEvent in queue, if any are present
	 * @throws ControllerException if the controller was stopped
	 */
	public WorkspaceEvent tryRecv() throws ControllerException {
		return try_recv(this.ptr);
	}

	private static native WorkspaceEvent recv(long self) throws ControllerException;

	/**
	 * Blocks until a {@link WorkspaceEvent} is available and returns it.
	 * @return the workspace WorkspaceEvent that occurred
	 * @throws ControllerException if the controller was stopped
	 */
	public WorkspaceEvent recv() throws ControllerException {
		return recv(this.ptr);
	}

	private static native void callback(long self, Consumer<Workspace> cb);

	/**
	 * Registers a callback to be invoked whenever a new {@link WorkspaceEvent} is ready to be received.
	 * This will not work unless a Java thread has been dedicated to the WorkspaceEvent loop.
	 * @param cb a {@link Consumer} that receives the controller when the change occurs;
	 *           you should probably spawn a new thread in here, to avoid deadlocking
	 * @see Extensions#drive(boolean)
	 */
	public void callback(Consumer<Workspace> cb) {
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
	 * Blocks until a {@link WorkspaceEvent} is available.
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
