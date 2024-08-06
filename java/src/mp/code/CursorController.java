package mp.code;

import mp.code.data.Cursor;
import mp.code.data.TextChange;

public class CursorController {
	private final long ptr;

	CursorController(long ptr) {
		this.ptr = ptr;
	}

	private static native Cursor try_recv(long self);
	public Cursor tryRecv() {
		return try_recv(this.ptr);
	}

	private static native void send(long self, Cursor cursor);
	public void send(TextChange change, Cursor cursor) {
		send(this.ptr, cursor);
	}

	private static native void free(long self);
	@Override
	@SuppressWarnings("removal")
	protected void finalize() throws Throwable {
		free(this.ptr);
	}
}
