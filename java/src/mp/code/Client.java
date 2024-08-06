package mp.code;

import mp.code.data.Cursor;
import mp.code.exceptions.CodeMPLibException;

import java.util.Optional;
import java.util.UUID;

public class Client {
	private final long ptr;
	private final String url;

	private static native long setup_tracing(String path);

	private static native long connect(String url) throws CodeMPLibException;
	public Client(String url) throws CodeMPLibException {
		this.ptr = connect(url);
		this.url = url;
	}

	public String getUrl() {
		return this.url;
	}

	private static native void login(long self, String username, String password, String workspace) throws CodeMPLibException;
	public void login(String username, String password, String workspace) throws CodeMPLibException {
		login(this.ptr, username, password, workspace);
	}

	private static native long join_workspace(long self, String id) throws CodeMPLibException;
	public Workspace joinWorkspace(String id) throws CodeMPLibException {
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

	// TODO - remove everything past this line
	public static void main(String[] args) throws CodeMPLibException, InterruptedException {
		Client c = new Client("http://alemi.dev:50053");
		c.login(UUID.randomUUID().toString(), "lmaodefaultpassword", "glue");
		Workspace workspace = c.joinWorkspace("glue");
		System.out.println(workspace.getWorkspaceId());
		while(true) {
			Cursor cursor = workspace.getCursor().tryRecv();
			if(cursor == null) System.out.println("null!");
			else {
				System.out.printf(
					"sr: %d, sc: %d, er: %d, ec: %d, cursor: %s, buffer: %s\n",
					cursor.startRow,
					cursor.startCol,
					cursor.endRow,
					cursor.endCol,
					cursor.user,
					cursor.buffer
				);
			}
			Thread.sleep(100);
		}

		//System.out.println("Done!");
	}
	
	static {
		System.loadLibrary("codemp");
		setup_tracing(null);
	}
}

