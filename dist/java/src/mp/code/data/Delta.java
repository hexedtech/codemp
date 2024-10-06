package mp.code.data;

import lombok.Getter;
import mp.code.data.Config;
import mp.code.data.User;
import mp.code.exceptions.ConnectionException;
import mp.code.exceptions.ConnectionRemoteException;

import java.util.Optional;

@Getter
public final class Delta {
	private final long ptr;

	Delta(long ptr) {
		this.ptr = ptr;
		Extensions.CLEANER.register(this, () -> free(ptr));
	}

	private static native TextChange get_text_change(long self);

	public mp.code.data.TextChange getTextChange() {
		return get_text_change(this.ptr);
	}

	private static native void ack_native(long self, boolean success) throws ConnectionException;

	public void ack(boolean success) throws ConnectionException {
		return ack_native(this.ptr, success);
	}

	private static native void free(long self);

	static {
		NativeUtils.loadLibraryIfNeeded();
	}
}
