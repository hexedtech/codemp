package mp.code;

import mp.code.data.Callback;
import mp.code.data.Cursor;
import mp.code.exceptions.ControllerException;

import java.util.Optional;

public class CursorController {
	private final long ptr;

	CursorController(long ptr) {
		this.ptr = ptr;
	}

	private static native Cursor try_recv(long self) throws ControllerException;
	public Optional<Cursor> tryRecv() throws ControllerException {
		return Optional.ofNullable(try_recv(this.ptr));
	}

	private static native Cursor recv(long self) throws ControllerException;
	public Cursor recv() throws ControllerException {
		return recv(this.ptr);
	}

	private static native void send(long self, Cursor cursor) throws ControllerException;
	public void send(Cursor cursor) throws ControllerException {
		send(this.ptr, cursor);
	}

	private static native void callback(long self, Callback<CursorController> cb);
	public void callback(Callback<CursorController> cb) {
		callback(this.ptr, cb);
	}

	private static native void clear_callback(long self);
	public void clearCallback() {
		clear_callback(this.ptr);
	}

	private static native void poll(long self);
	public void poll() {
		poll(this.ptr);
	}

	private static native boolean stop(long self);
	public boolean stop() {
		return stop(this.ptr);
	}

	private static native void free(long self);
	@Override
	protected void finalize() {
		free(this.ptr);
	}

	static {
		Extensions.loadLibraryIfNotPresent();
	}
}
