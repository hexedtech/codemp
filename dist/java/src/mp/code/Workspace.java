package mp.code;

import java.util.Optional;
import java.util.UUID;

import mp.code.data.DetachResult;
import mp.code.exceptions.ConnectionException;
import mp.code.exceptions.ConnectionRemoteException;
import mp.code.exceptions.ControllerException;

public class Workspace {
	private final long ptr;

	Workspace(long ptr) {
		this.ptr = ptr;
	}

	private static native String get_workspace_id(long self);
	public String getWorkspaceId() {
		return get_workspace_id(this.ptr);
	}

	private static native CursorController get_cursor(long self);
	public CursorController getCursor() {
		return get_cursor(this.ptr);
	}

	private static native BufferController get_buffer(long self, String path);
	public Optional<BufferController> getBuffer(String path) {
		return Optional.ofNullable(get_buffer(this.ptr, path));
	}

	private static native String[] get_file_tree(long self, String filter, boolean strict);
	public String[] getFileTree(Optional<String> filter, boolean strict) {
		return get_file_tree(this.ptr, filter.orElse(null), strict);
	}

	private static native void create_buffer(String path) throws ConnectionRemoteException;
	public void createBuffer(String path) throws ConnectionRemoteException {
		create_buffer(path);
	}

	private static native BufferController attach_to_buffer(long self, String path) throws ConnectionException;
	public BufferController attachToBuffer(String path) throws ConnectionException {
		return attach_to_buffer(ptr, path);
	}

	private static native DetachResult detach_from_buffer(long self, String path);
	public DetachResult detachFromBuffer(String path) {
		return detach_from_buffer(this.ptr, path);
	}

	private static native void fetch_buffers(long self) throws ConnectionRemoteException;
	public void fetchBuffers() throws ConnectionRemoteException {
		fetch_buffers(this.ptr);
	}

	private static native void fetch_users(long self) throws ConnectionRemoteException;
	public void fetchUsers() throws ConnectionRemoteException {
		fetch_buffers(this.ptr);
	}

	private static native UUID[] list_buffer_users(long self, String path) throws ConnectionRemoteException;
	public UUID[] listBufferUsers(String path) throws ConnectionRemoteException {
		return list_buffer_users(this.ptr, path);
	}

	private static native void delete_buffer(long self, String path) throws ConnectionRemoteException;
	public void deleteBuffer(String path) throws ConnectionRemoteException {
		delete_buffer(this.ptr, path);
	}

	private static native Event event(long self) throws ControllerException;
	public Event event() throws ControllerException {
		return event(this.ptr);
	}

	private static native BufferController select_buffer(long self, long timeout) throws ControllerException;
	public Optional<BufferController> selectBuffer(long timeout) throws ControllerException {
		return Optional.ofNullable(select_buffer(this.ptr, timeout));
	}

	private static native void free(long self);
	@Override
	protected void finalize() {
		free(this.ptr);
	}
	
	public static class Event {
		private final Type type;
		private final String argument;

		Event(Type type, String argument) {
			this.type = type;
			this.argument = argument;
		}

		public Optional<String  > getUserJoined() {
			if(this.type == Type.USER_JOIN) {
				return Optional.of(this.argument);
			} else return Optional.empty();
		}

		public Optional<String> getUserLeft() {
			if(this.type == Type.USER_LEAVE) {
				return Optional.of(this.argument);
			} else return Optional.empty();
		}

		public Optional<String> getTargetBuffer() {
			if(this.type == Type.FILE_TREE_UPDATED) {
				return Optional.of(this.argument);
			} else return Optional.empty();
		}

		private enum Type {
			USER_JOIN,
			USER_LEAVE,
			FILE_TREE_UPDATED
		}
	}
}
