package mp.code;

import java.util.Optional;
import java.util.UUID;

import mp.code.data.DetachResult;
import mp.code.exceptions.CodeMPException;

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

	private static native String[] get_file_tree(long self);
	public String[] getFileTree() {
		return get_file_tree(this.ptr);
	}

	private static native long create_buffer(String path) throws CodeMPException;
	public BufferController createBuffer(String path) throws CodeMPException {
		return new BufferController(create_buffer(path));
	}

	private static native BufferController attach_to_buffer(long self, String path) throws CodeMPException;
	public BufferController attachToBuffer(String path) throws CodeMPException {
		return attach_to_buffer(ptr, path);
	}

	private static native DetachResult detach_from_buffer(long self, String path);
	public DetachResult detachFromBuffer(String path) {
		return detach_from_buffer(this.ptr, path);
	}

	private static native void fetch_buffers(long self) throws CodeMPException;
	public void fetchBuffers() throws CodeMPException {
		fetch_buffers(this.ptr);
	}

	private static native void fetch_users(long self) throws CodeMPException;
	public void fetchUsers() throws CodeMPException {
		fetch_buffers(this.ptr);
	}

	private static native UUID[] list_buffer_users(long self, String path) throws CodeMPException;
	public UUID[] listBufferUsers(String path) throws CodeMPException {
		return list_buffer_users(this.ptr, path);
	}

	private static native void delete_buffer(long self, String path) throws CodeMPException;
	public void deleteBuffer(String path) throws CodeMPException {
		delete_buffer(this.ptr, path);
	}

	private static native Event event(long self) throws CodeMPException;
	public Event event() throws CodeMPException {
		return event(this.ptr);
	}

	private static native BufferController select_buffer(long self, long timeout) throws CodeMPException;
	public Optional<BufferController> selectBuffer(long timeout) throws CodeMPException {
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

		public Optional<String> getUserJoined() {
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
