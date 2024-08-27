package mp.code;

import cz.adamh.utils.NativeUtils;
import mp.code.exceptions.CodeMPException;

import java.io.IOException;
import java.util.Optional;

public class Client {
	private final long ptr;

	public static native Client connect(String url, String username, String password) throws CodeMPException;
	Client(long ptr) {
		this.ptr = ptr;
	}

	private static native String get_url(long self);
	public String getUrl() {
		return get_url(this.ptr);
	}

	private static native Workspace join_workspace(long self, String id) throws CodeMPException;
	public Workspace joinWorkspace(String id) throws CodeMPException {
		return join_workspace(this.ptr, id);
	}

	private static native void create_workspace(long self, String id) throws CodeMPException;
	public void createWorkspace(String id) throws CodeMPException {
		return create_workspace(this.ptr, id);
	}

	private static native void delete_workspace(long self, String id) throws CodeMPException;
	public void deleteWorkspace(String id) throws CodeMPException {
		return delete_workspace(this.ptr, id);
	}

	private static native void invite_to_workspace(long self, String ws, String usr) throws CodeMPException;
	public void inviteToWorkspace(String ws, String usr) throws CodeMPException {
		return invite_to_workspace(this.ptr, id);
	}

	private static native String[] list_workspaces(long self, boolean owned, boolean invited) throws CodeMPException;
	public String[] listWorkspaces(boolean owned, boolean invited) throws CodeMPException {
		return list_workspaces(this.ptr, owned, invited);
	}

	private static native boolean leave_workspace(long self, String id);
	public boolean leaveWorkspace(String id) {
		return leave_workspace(this.ptr, id);
	}

	private static native Workspace get_workspace(long self);
	public Optional<Workspace> getWorkspace() {
		return Optional.ofNullable(get_workspace(this.ptr));
	}

	private static native void refresh_native(long self);
	public void refresh() {
		return refresh_native(this.ptr);
	}
	
	private static native void free(long self);
	@Override
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

