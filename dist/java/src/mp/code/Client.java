package mp.code;

import lombok.Getter;
import mp.code.data.Config;
import mp.code.data.User;
import mp.code.exceptions.ConnectionException;
import mp.code.exceptions.ConnectionRemoteException;

import java.util.Optional;

/**
 * The main entrypoint of the library.
 * This is the only object you are expected to hold yourself; unlike all the others,
 * there are no copies of it managed exclusively by the library. When this is garbage
 * collected, it will free the underlying memory.
 * A Client is used to join and manage workspaces, and to obtain information about
 * the current session.
 */
@Getter
public final class Client {
	private final long ptr;

	Client(long ptr) {
		this.ptr = ptr;
	}

	/**
	 * Connects to a remote CodeMP server and creates a {@link Client} instance
	 * for interacting with it.
	 * @param config a {@link Config} object containing the connection settings
	 * @return a holder for the Client's pointer
	 * @throws ConnectionException if an error occurs in communicating with the server
	 */
	public static native Client connect(Config config) throws ConnectionException;

	private static native User get_user(long self);

	/**
	 * Gets information about the current user.
	 * @return a {@link User} object representing the user
	 */
	public User getUser() {
		return get_user(this.ptr);
	}

	private static native Workspace join_workspace(long self, String workspaceId) throws ConnectionException;

	/**
	 * Joins a {@link Workspace} and returns it.
	 * @param workspaceId the id of the workspace to connect to
	 * @return the relevant {@link Workspace}
	 * @throws ConnectionException if an error occurs in communicating with the server
	 */
	public Workspace joinWorkspace(String workspaceId) throws ConnectionException {
		return join_workspace(this.ptr, workspaceId);
	}

	private static native void create_workspace(long self, String workspaceId) throws ConnectionRemoteException;

	/**
	 * Creates a workspace. You need to call {@link #joinWorkspace(String)} to actually join
	 * and interact with it.
	 * @param workspaceId the id of the new workspace
	 * @throws ConnectionRemoteException if an error occurs in communicating with the server
	 */
	public void createWorkspace(String workspaceId) throws ConnectionRemoteException {
		create_workspace(this.ptr, workspaceId);
	}

	private static native void delete_workspace(long self, String workspaceId) throws ConnectionRemoteException;

	/**
	 * Deletes a workspace.
	 * @param workspaceId the id of the workspace to delete
	 * @throws ConnectionRemoteException if an error occurs in communicating with the server
	 */
	public void deleteWorkspace(String workspaceId) throws ConnectionRemoteException {
		delete_workspace(this.ptr, workspaceId);
	}

	private static native void invite_to_workspace(long self, String workspaceId, String user) throws ConnectionRemoteException;

	/**
	 * Invites a user to a workspace.
	 * @param workspaceId the id of the new workspace
	 * @param user the name of the user to invite
	 * @throws ConnectionRemoteException if an error occurs in communicating with the server
	 */
	public void inviteToWorkspace(String workspaceId, String user) throws ConnectionRemoteException {
		invite_to_workspace(this.ptr, workspaceId, user);
	}

	private static native String[] list_workspaces(long self, boolean owned, boolean invited) throws ConnectionRemoteException;

	/**
	 * Lists available workspaces according to certain filters.
	 * @param owned if owned workspaces should be included
	 * @param invited if workspaces the user is invited to should be included
	 * @return an array of workspace IDs
	 * @throws ConnectionRemoteException if an error occurs in communicating with the server
	 */
	public String[] listWorkspaces(boolean owned, boolean invited) throws ConnectionRemoteException {
		return list_workspaces(this.ptr, owned, invited);
	}

	private static native String[] active_workspaces(long self);

	/**
	 * Lists the currently active workspaces (the ones the user has currently joined).
	 * @return an array of workspace IDs
	 */
	public String[] activeWorkspaces() {
		return active_workspaces(this.ptr);
	}

	private static native boolean leave_workspace(long self, String workspaceId);

	/**
	 * Leaves a workspace.
	 * @param workspaceId the id of the workspaces to leave
	 * @return true if it succeeded (usually fails if the workspace wasn't active)
	 */
	public boolean leaveWorkspace(String workspaceId) {
		return leave_workspace(this.ptr, workspaceId);
	}

	private static native Workspace get_workspace(long self, String workspace);

	/**
	 * Gets an active workspace.
	 * @param workspaceId the id of the workspaces to get
	 * @return a {@link Workspace} with that name, if it was present and active
	 */
	public Optional<Workspace> getWorkspace(String workspaceId) {
		return Optional.ofNullable(get_workspace(this.ptr, workspaceId));
	}

	private static native void refresh(long self) throws ConnectionRemoteException;

	/**
	 * Refreshes the current access token.
	 * @throws ConnectionRemoteException if an error occurs in communicating with the server
	 */
	public void refresh() throws ConnectionRemoteException {
		refresh(this.ptr);
	}

	private static native void free(long self);
	@Override
	protected void finalize() {
		free(this.ptr);
	}

	static {
		NativeUtils.loadLibraryIfNeeded();
	}
}
