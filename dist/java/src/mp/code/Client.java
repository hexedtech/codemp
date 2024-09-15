package mp.code;

import cz.adamh.utils.NativeUtils;
import mp.code.data.User;
import mp.code.exceptions.ConnectionException;
import mp.code.exceptions.ConnectionRemoteException;

import java.io.IOException;
import java.util.Optional;

public class Client {
	private final long ptr;

	public static native Client connect(String username, String password) throws ConnectionException;
	public static native Client connectToServer(String username, String password, String host, int port, boolean tls) throws ConnectionException;

	Client(long ptr) {
		this.ptr = ptr;
	}

	private static native User get_user(long self);
	public User getUser() {
		return get_user(this.ptr);
	}

	private static native Workspace join_workspace(long self, String id) throws ConnectionException;
	public Workspace joinWorkspace(String id) throws ConnectionException {
		return join_workspace(this.ptr, id);
	}

	private static native void create_workspace(long self, String id) throws ConnectionRemoteException;
	public void createWorkspace(String id) throws ConnectionRemoteException {
		create_workspace(this.ptr, id);
	}

	private static native void delete_workspace(long self, String id) throws ConnectionRemoteException;
	public void deleteWorkspace(String id) throws ConnectionRemoteException {
		delete_workspace(this.ptr, id);
	}

	private static native void invite_to_workspace(long self, String ws, String usr) throws ConnectionRemoteException;
	public void inviteToWorkspace(String ws, String usr) throws ConnectionRemoteException {
		invite_to_workspace(this.ptr, ws, usr);
	}

	private static native String[] list_workspaces(long self, boolean owned, boolean invited) throws ConnectionRemoteException;
	public String[] listWorkspaces(boolean owned, boolean invited) throws ConnectionRemoteException {
		return list_workspaces(this.ptr, owned, invited);
	}

	private static native String[] active_workspaces(long self);
	public String[] activeWorkspaces() {
		return active_workspaces(this.ptr);
	}

	private static native boolean leave_workspace(long self, String id);
	public boolean leaveWorkspace(String id) {
		return leave_workspace(this.ptr, id);
	}

	private static native Workspace get_workspace(long self, String workspace);
	public Optional<Workspace> getWorkspace(String workspace) {
		return Optional.ofNullable(get_workspace(this.ptr, workspace));
	}

	private static native void refresh(long self) throws ConnectionRemoteException;
	public void refresh() throws ConnectionRemoteException {
		refresh(this.ptr);
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
				NativeUtils.loadLibraryFromJar("/natives/codemp.dll");
			else NativeUtils.loadLibraryFromJar("/natives/libcodemp.so");
			setup_tracing(System.getenv().get("CODEMP_TRACING_LOG"));
		} catch(IOException e) {
			throw new RuntimeException(e);
		}
	}
}

