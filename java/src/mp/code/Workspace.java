package mp.code;

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

	private static native long get_cursor(long self);
	public CursorController getCursor() {
		return new CursorController(get_cursor(this.ptr));
	}

	private static native long get_buffer(long self, String path);
	public BufferController getBuffer(String path) {
		return new BufferController(get_buffer(this.ptr, path));
	}

	private static native String[] get_file_tree(long self);
	public String[] getFileTree() {
		return get_file_tree(this.ptr);
	}

	private static native long create_buffer(String path) throws CodeMPException;
	public BufferController createBuffer(String path) throws CodeMPException {
		return new BufferController(create_buffer(path));
	}

	private static native long attach_to_buffer(long self) throws CodeMPException;
	public BufferController attachToBuffer() throws CodeMPException {
		return new BufferController(attach_to_buffer(ptr));
	}

	private static native void fetch_buffers(long self) throws CodeMPException;
	public void fetchBuffers() throws CodeMPException {
		fetch_buffers(this.ptr);
	}

	private static native void fetch_users(long self) throws CodeMPException;
	public void fetchUsers() throws CodeMPException {
		fetch_buffers(this.ptr);
	}

	private static native String[] list_buffer_users(long self, String path) throws CodeMPException;
	public String[] listBufferUsers(String path) throws CodeMPException {
		return list_buffer_users(this.ptr, path);
	}

			private static native void delete_buffer(long self, String path) throws CodeMPException;
	public void deleteBuffer(String path) throws CodeMPException {
		delete_buffer(this.ptr, path);
	}

	private static native BufferController select_buffer(long self, long timeout) throws CodeMPException;
	public BufferController selectBuffer(long timeout) throws CodeMPException {
		return select_buffer(this.ptr, timeout);
	}

	private static native void free(long self);
	@Override
	@SuppressWarnings("removal")
	protected void finalize() {
		free(this.ptr);
	}
}
