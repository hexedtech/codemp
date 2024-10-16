package mp.code;

import java.util.Optional;
import java.util.UUID;
import java.util.function.Consumer;

import mp.code.exceptions.ConnectionException;
import mp.code.exceptions.ConnectionRemoteException;
import mp.code.exceptions.ControllerException;

/**
 * Represents a CodeMP workspace, which broadly speaking is a collection
 * of buffers across which edits and cursor movements are tracked.
 * Generally, it is safer to avoid storing this directly. Instead,
 * users should let the native library manage as much as possible for
 * them. They should store the workspace ID and retrieve the object
 * whenever needed with {@link Client#getWorkspace(String)}.
 */
public final class Workspace {
	private final long ptr;

	Workspace(long ptr) {
		this.ptr = ptr;
		Extensions.CLEANER.register(this, () -> free(ptr));
	}

	private static native String get_workspace_id(long self);

	/**
	 * Gets the unique identifier of the current workspace.
	 * @return the identifier
	 */
	public String getWorkspaceId() {
		return get_workspace_id(this.ptr);
	}

	private static native CursorController get_cursor(long self);

	/**
	 * Gets the {@link CursorController} for the current workspace.
	 * @return the {@link CursorController}
	 */
	public CursorController getCursor() {
		return get_cursor(this.ptr);
	}

	private static native BufferController get_buffer(long self, String path);

	/**
	 * Looks for a {@link BufferController} with the given path within the
	 * current workspace and returns it if it exists.
	 * @param path the current path
	 * @return the {@link BufferController} with the given path, if it exists
	 */
	public Optional<BufferController> getBuffer(String path) {
		return Optional.ofNullable(get_buffer(this.ptr, path));
	}

	private static native String[] get_file_tree(long self, String filter, boolean strict);

	/**
	 * Gets the file tree for this workspace, optionally filtering it.
	 * @param filter applies an optional filter to the outputs
	 * @param strict whether it should be a strict match (equals) or not (startsWith)
	 * @return an array containing file tree as flat paths
	 */
	@SuppressWarnings("OptionalUsedAsFieldOrParameterType")
	public String[] getFileTree(Optional<String> filter, boolean strict) {
		return get_file_tree(this.ptr, filter.orElse(null), strict);
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

	private static native String[] user_list(long self);

	/**
	 * Returns the users currently in the workspace.
	 * @return an array containing the names of the users in the workspace
	 */
	public String[] userList() {
		return user_list(this.ptr);
	}

	private static native void create_buffer(long self, String path) throws ConnectionRemoteException;

	/**
	 * Creates a buffer with the given path.
	 * @param path the new buffer's path
	 * @throws ConnectionRemoteException if an error occurs in communicating with the server
	 */
	public void createBuffer(String path) throws ConnectionRemoteException {
		create_buffer(this.ptr, path);
	}

	private static native BufferController attach_to_buffer(long self, String path) throws ConnectionException;

	/**
	 * Attaches to an existing buffer with the given path, if present.
	 * @param path the path of the buffer to attach to
	 * @return the {@link BufferController} associated with that path
	 * @throws ConnectionException if an error occurs in communicating with the server, or if the buffer did not exist
	 */
	public BufferController attachToBuffer(String path) throws ConnectionException {
		return attach_to_buffer(ptr, path);
	}

	private static native boolean detach_from_buffer(long self, String path);

	/**
	 * Detaches from a given buffer.
	 * @param path the path of the buffer to detach from
	 * @return a boolean, true only if there are still dangling references preventing controller from stopping
	 */
	public boolean detachFromBuffer(String path) {
		return detach_from_buffer(this.ptr, path);
	}

	private static native void fetch_buffers(long self) throws ConnectionRemoteException;

	/**
	 * Updates the local list of buffers.
	 * @throws ConnectionRemoteException if an error occurs in communicating with the server
	 */
	public void fetchBuffers() throws ConnectionRemoteException {
		fetch_buffers(this.ptr);
	}

	private static native void fetch_users(long self) throws ConnectionRemoteException;

	/**
	 * Updates the local list of users.
	 * @throws ConnectionRemoteException if an error occurs in communicating with the server
	 */
	public void fetchUsers() throws ConnectionRemoteException {
		fetch_buffers(this.ptr);
	}

	private static native UUID[] list_buffer_users(long self, String path) throws ConnectionRemoteException;

	/**
	 * Lists the user attached to a certain buffer.
	 * The user must be attached to the buffer to perform this operation.
	 * @param path the path of the buffer to search
	 * @return an array of user {@link UUID UUIDs}
	 * @throws ConnectionRemoteException if an error occurs in communicating with the server, or the user wasn't attached
	 */
	public UUID[] listBufferUsers(String path) throws ConnectionRemoteException {
		return list_buffer_users(this.ptr, path);
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

	private static native Event try_recv(long self) throws ControllerException;

	/**
	 * Tries to get a {@link Event} from the queue if any were present, and returns
	 * an empty optional otherwise.
	 * @return the first workspace event in queue, if any are present
	 * @throws ControllerException if the controller was stopped
	 */
	public Optional<Event> tryRecv() throws ControllerException {
		return Optional.ofNullable(try_recv(this.ptr));
	}

	private static native Event recv(long self) throws ControllerException;

	/**
	 * Blocks until a {@link Event} is available and returns it.
	 * @return the workspace event that occurred
	 * @throws ControllerException if the controller was stopped
	 */
	public Event recv() throws ControllerException {
		return recv(this.ptr);
	}

	private static native void callback(long self, Consumer<Workspace> cb);

	/**
	 * Registers a callback to be invoked whenever a new {@link Event} is ready to be received.
	 * This will not work unless a Java thread has been dedicated to the event loop.
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
	 * Blocks until a {@link Event} is available.
	 * @throws ControllerException if the controller was stopped
	 */
	public void poll() throws ControllerException {
		poll(this.ptr);
	}

	private static native void free(long self);

	static {
		NativeUtils.loadLibraryIfNeeded();
	}

	/**
	 * Represents a workspace-wide event.
	 */
	public static final class Event {
		private final Type type;
		private final String argument;

		Event(Type type, String argument) {
			this.type = type;
			this.argument = argument;
		}

		/**
		 * Gets the user who joined, if any did.
		 * @return the user who joined, if any did
		 */
		public Optional<String> getUserJoined() {
			if(this.type == Type.USER_JOIN) {
				return Optional.of(this.argument);
			} else return Optional.empty();
		}

		/**
		 * Gets the user who left, if any did.
		 * @return the user who left, if any did
		 */
		public Optional<String> getUserLeft() {
			if(this.type == Type.USER_LEAVE) {
				return Optional.of(this.argument);
			} else return Optional.empty();
		}

		/**
		 * Gets the path of buffer that changed, if any did.
		 * @return the path of buffer that changed, if any did
		 */
		public Optional<String> getChangedBuffer() {
			if(this.type == Type.FILE_TREE_UPDATED) {
				return Optional.of(this.argument);
			} else return Optional.empty();
		}

		enum Type {
			USER_JOIN,
			USER_LEAVE,
			FILE_TREE_UPDATED
		}
	}
}
