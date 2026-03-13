package mp.code;

import lombok.Getter;
import mp.code.data.Config;
import mp.code.data.UserInfo;
import mp.code.data.WorkspaceIdentifier;
import mp.code.exceptions.ConnectionException;
import mp.code.exceptions.ConnectionRemoteException;

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
		Extensions.CLEANER.register(this, () -> free(ptr));
	}

	/**
	 * Connects to a remote CodeMP server and creates a {@link Client} instance
	 * for interacting with it.
	 * @param config a {@link Config} object containing the connection settings
	 * @return a holder for the Client's pointer
	 * @throws ConnectionException if an error occurs in communicating with the server
	 */
	public static native Client connect(Config config) throws ConnectionException;

	private static native UserInfo current_user(long self);

	/**
	 * Gets information about the current user.
	 * @return a {@link UserInfo} object representing the user
	 */
	public UserInfo currentUser() {
		return current_user(this.ptr);
	}

	private static native Workspace attach_workspace(long self, String user, String workspace) throws ConnectionException;

	/**
	 * Joins a {@link Workspace} and returns it.
	 * @param user the owner of the workspace
	 * @param workspace the identifier of the workspace
	 * @return the relevant {@link Workspace}
	 * @throws ConnectionException if an error occurs in communicating with the server
	 */
	public Workspace attachWorkspace(String user, String workspace) throws ConnectionException {
		return attach_workspace(this.ptr, user, workspace);
	}

	private static native void accept_invite(long self, String user, String workspace) throws ConnectionRemoteException;

	/**
	 * Accept an invitation to a workspace.
	 * @param user the owner of the workspace
	 * @param workspace the identifier of the workspace
	 * @throws ConnectionRemoteException if an error occurs in communicating with the server
	 */
	public void acceptInvite(String user, String workspace) throws ConnectionRemoteException {
		accept_invite(this.ptr, user, workspace);
	}

	private static native void reject_invite(long self, String user, String workspace) throws ConnectionRemoteException;

	/**
	 * Rejects an invitation to a workspace
	 * @param user the owner of the workspace
	 * @param workspace the identifier of the workspace
	 * @throws ConnectionRemoteException if an error occurs in communicating with the server
	 */
	public void rejectInvite(String user, String workspace) throws ConnectionRemoteException {
		reject_invite(this.ptr, user, workspace);
	}

	private static native void quit_workspace(long self, String user, String workspace) throws ConnectionRemoteException;

	/**
	 * Quits a workspace.
	 * @param user the owner of the workspace
	 * @param workspace the identifier of the workspace
	 * @throws ConnectionRemoteException if an error occurs in communicating with the server
	 */
	public void quitWorkspace(String user, String workspace) throws ConnectionRemoteException {
		quit_workspace(this.ptr, user, workspace);
	}

	private static native void create_workspace(long self, String workspace) throws ConnectionRemoteException;

	/**
	 * Creates a workspace. You need to call {@link #attachWorkspace(String, String)} to actually join
	 * and interact with it.
	 * @param workspace the id of the new workspace
	 * @throws ConnectionRemoteException if an error occurs in communicating with the server
	 */
	public void createWorkspace(String workspace) throws ConnectionRemoteException {
		create_workspace(this.ptr, workspace);
	}

	private static native void delete_workspace(long self, String workspace) throws ConnectionRemoteException;

	/**
	 * Deletes a workspace.
	 * @param workspace the id of the workspace to delete
	 * @throws ConnectionRemoteException if an error occurs in communicating with the server
	 */
	public void deleteWorkspace(String workspace) throws ConnectionRemoteException {
		delete_workspace(this.ptr, workspace);
	}

	private static native void invite_to_workspace(long self, String workspaceId, String user) throws ConnectionRemoteException;

	/**
	 * Invites a user to a workspace.
	 * @param workspace the id of the new workspace
	 * @param user the name of the user to invite
	 * @throws ConnectionRemoteException if an error occurs in communicating with the server
	 */
	public void inviteToWorkspace(String workspace, String user) throws ConnectionRemoteException {
		invite_to_workspace(this.ptr, workspace, user);
	}

	private static native WorkspaceIdentifier[] fetch_owned_workspaces(long self) throws ConnectionRemoteException;

	/**
	 * Lists workspaces owned by the current user.
	 * @return an array of {@link WorkspaceIdentifier}s
	 * @throws ConnectionRemoteException if an error occurs in communicating with the server
	 */
	public WorkspaceIdentifier[] fetchOwnedWorkspaces() throws ConnectionRemoteException {
		return fetch_owned_workspaces(this.ptr);
	}

	private static native WorkspaceIdentifier[] fetch_joined_workspaces(long self) throws ConnectionRemoteException;

	/**
	 * Lists workspaces the current user has joined.
	 * @return an array of {@link WorkspaceIdentifier}s
	 * @throws ConnectionRemoteException if an error occurs in communicating with the server
	 */
	public WorkspaceIdentifier[] fetchJoinedWorkspaces() throws ConnectionRemoteException {
		return fetch_joined_workspaces(this.ptr);
	}

	private static native WorkspaceIdentifier[] active_workspaces(long self);

	/**
	 * Lists the currently active workspaces (the ones the user has currently joined).
	 * @return an array of {@link WorkspaceIdentifier}s
	 */
	public WorkspaceIdentifier[] activeWorkspaces() {
		return active_workspaces(this.ptr);
	}

	private static native boolean leave_workspace(long self, String user, String workspace);

	/**
	 * Leaves a workspace.
	 * @param user the owner of the workspace
	 * @param workspace the identifier of the workspace
	 * @return true if it succeeded or wasn't in the workspace; false if there are still
	 *         leftover references around
	 */
	public boolean leaveWorkspace(String user, String workspace) {
		return leave_workspace(this.ptr, user, workspace);
	}

	private static native Workspace get_workspace(long self, String user, String workspace);

	/**
	 * Gets an active workspace.
	 * @param user the owner of the workspace
	 * @param workspace the identifier of the workspace
	 * @return a {@link Workspace} with that name, if it was present and active, null otherwise
	 */
	public Workspace getWorkspace(String user, String workspace) {
		return get_workspace(this.ptr, user, workspace);
	}

	private static native UserInfo get_user_info(long self, String user) throws ConnectionRemoteException;

	/**
	 * Fetches information about a user by name.
	 * @param user the name of the user
	 * @return the {@link UserInfo} for the user
	 * @throws ConnectionRemoteException if an error occurs in communicating with the server
	 */
	public UserInfo getUserInfo(String user) throws ConnectionRemoteException {
		return get_user_info(this.ptr, user);
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

	static {
		NativeUtils.loadLibraryIfNeeded();
	}
}
