package mp.code;

import mp.code.data.Cursor;
import mp.code.exceptions.CodeMPException;

import java.util.Optional;

public class CursorController {
	private final long ptr;

	CursorController(long ptr) {
		this.ptr = ptr;
	}

	private static native Cursor try_recv(long self) throws CodeMPException;
	public Optional<Cursor> tryRecv() throws CodeMPException {
		return Optional.ofNullable(try_recv(this.ptr));
	}

	private static native Cursor recv(long self) throws CodeMPException;
	public Cursor recv() throws CodeMPException {
		return recv(this.ptr);
	}

	private static native void send(long self, Cursor cursor) throws CodeMPException;
	public void send(Cursor cursor) throws CodeMPException {
		send(this.ptr, cursor);
	}

	private static native void free(long self);
	@Override
	protected void finalize() {
		free(this.ptr);
	}
}
