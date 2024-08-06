package mp.code;

import mp.code.exceptions.CodeMPLibException;

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

	private static native void get_file_tree(long self);
	public void getFileTree() {
		// TODO vector?
	}

	private static native long create_buffer(String path) throws CodeMPLibException;
	public BufferController createBuffer(String path) throws CodeMPLibException {
		return new BufferController(create_buffer(path));
	}

	private static native long attach_to_buffer(long self) throws CodeMPLibException;
	public BufferController attachToBuffer() throws CodeMPLibException {
		return new BufferController(attach_to_buffer(ptr));
	}

	private static native void fetch_buffers(long self) throws CodeMPLibException;
	public void fetchBuffers() throws CodeMPLibException {
		fetch_buffers(this.ptr);
	}

	private static native void fetch_users(long self) throws CodeMPLibException;
	public void fetchUsers() throws CodeMPLibException {
		fetch_buffers(this.ptr);
	}

	private static native void list_buffer_users(long self, String path) throws CodeMPLibException;
	public void listBufferUsers(String path) throws CodeMPLibException {
		// TODO pass vector
	}

	private static native void delete_buffer(long self, String path) throws CodeMPLibException;
	public void deleteBuffer(String path) throws CodeMPLibException {
		delete_buffer(this.ptr, path);
	}

	// TODO select_buffer

	private static native void free(long self);
	@Override
	@SuppressWarnings("removal")
	protected void finalize() throws Throwable {
		free(this.ptr);
	}
}
