package mp.code;

import mp.code.data.Cursor;
import mp.code.data.TextChange;
import mp.code.exceptions.CodeMPException;

import java.util.Optional;

public class BufferController {
	private final long ptr;

	BufferController(long ptr) {
		this.ptr = ptr;
	}

	public static native String get_name(long self);
	public String getName() {
		return get_name(this.ptr);
	}

	public static native String get_content(long self);
	public String getContent() {
		return get_content(this.ptr);
	}

	private static native TextChange try_recv(long self) throws CodeMPException;
	public Optional<TextChange> tryRecv() throws CodeMPException {
		return Optional.ofNullable(try_recv(this.ptr));
	}

	private static native Cursor recv(long self) throws CodeMPException;
	public Cursor recv() throws CodeMPException {
		return recv(this.ptr);
	}

	private static native void send(long self, TextChange change) throws CodeMPException;
	public void send(TextChange change) throws CodeMPException {
		send(this.ptr, change);
	}

	private static native void free(long self);
	@Override
	@SuppressWarnings("removal")
	protected void finalize() {
		free(this.ptr);
	}
}
