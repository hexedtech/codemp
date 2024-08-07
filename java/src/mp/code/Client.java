package mp.code;

import cz.adamh.utils.NativeUtils;
import mp.code.exceptions.CodeMPException;

import java.io.IOException;
import java.util.Optional;

public class Client {
	private final long ptr;

	public static native Client connect(String url) throws CodeMPException;
	Client(long ptr) {
		this.ptr = ptr;
	}

	private static native String get_url(long self);
	public String getUrl() {
		return get_url(this.ptr);
	}

	private static native void login(long self, String username, String password, String workspace) throws CodeMPException;
	public void login(String username, String password, String workspace) throws CodeMPException {
		login(this.ptr, username, password, workspace);
	}

	private static native Workspace join_workspace(long self, String id) throws CodeMPException;
	public Workspace joinWorkspace(String id) throws CodeMPException {
		return join_workspace(this.ptr, id);
	}

	private static native Workspace get_workspace(long self);
	public Optional<Workspace> getWorkspace() {
		return Optional.ofNullable(get_workspace(this.ptr));
	}
	
	private static native void free(long self);
	@Override
	@SuppressWarnings("removal") // muh java 8
	protected void finalize() {
		free(this.ptr);
	}

	private static native void setup_tracing(String path);
	static {
		try {
			if(System.getProperty("os.name").startsWith("Windows"))
				NativeUtils.loadLibraryFromJar("/natives/codemp_intellij.dll");
			else NativeUtils.loadLibraryFromJar("/natives/libcodemp_intellij.so");
			setup_tracing(System.getenv().get("CODEMP_TRACING_LOG"));
		} catch(IOException e) {
			throw new RuntimeException(e);
		}
	}
}

