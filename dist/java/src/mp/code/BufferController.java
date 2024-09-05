package mp.code;

import mp.code.data.Cursor;
import mp.code.data.TextChange;
import mp.code.exceptions.ControllerException;

import java.util.Optional;

public class BufferController {
	private final long ptr;

	BufferController(long ptr) {
		this.ptr = ptr;
	}

	private static native String get_name(long self);
	public String getName() {
		return get_name(this.ptr);
	}

	private static native String get_content(long self) throws ControllerException;
	public String getContent() throws ControllerException {
		return get_content(this.ptr);
	}

	private static native TextChange try_recv(long self) throws ControllerException;
	public Optional<TextChange> tryRecv() throws ControllerException {
		return Optional.ofNullable(try_recv(this.ptr));
	}

	private static native Cursor recv(long self) throws ControllerException;
	public Cursor recv() throws ControllerException {
		return recv(this.ptr);
	}

	private static native void send(long self, TextChange change) throws ControllerException;
	public void send(TextChange change) throws ControllerException {
		send(this.ptr, change);
	}

	private static native void free(long self);
	@Override
	protected void finalize() {
		free(this.ptr);
	}
}
