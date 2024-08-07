package mp.code;

import mp.code.exceptions.CodeMPException;

import java.util.Optional;

public class Client {
	private final long ptr;
	private final String url;

	public static native long setup_tracing(String path);

	private static native long connect(String url) throws CodeMPException;
	public Client(String url) throws CodeMPException {
		this.ptr = connect(url);
		this.url = url;
	}

	public String getUrl() {
		return this.url;
	}

	private static native void login(long self, String username, String password, String workspace) throws CodeMPException;
	public void login(String username, String password, String workspace) throws CodeMPException {
		login(this.ptr, username, password, workspace);
	}

	private static native long join_workspace(long self, String id) throws CodeMPException;
	public Workspace joinWorkspace(String id) throws CodeMPException {
		return new Workspace(join_workspace(this.ptr, id));
	}

	private static native long get_workspace(long self);
	public Optional<Workspace> getWorkspace() {
		long ptr = get_workspace(this.ptr);
		if(ptr == 0) { // TODO it would be better to init in rust directly
			return Optional.empty();
		} else {
			return Optional.of(new Workspace(ptr));
		}
	}
	
	private static native void free(long self);
	@Override
	@SuppressWarnings("removal") // muh java 8
	protected void finalize() {
		free(this.ptr);
	}
}

