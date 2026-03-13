package mp.code.data;

import lombok.EqualsAndHashCode;
import lombok.RequiredArgsConstructor;
import lombok.ToString;

/**
 * The unique identifier of a workspace.
 */
@ToString
@EqualsAndHashCode
@RequiredArgsConstructor
public class WorkspaceIdentifier {
	/**
	 * The workspace name, cannot change and is guaranteed to be unique per owner.
	 */
	public final String workspace;

	/**
	 * The workspace's owner.
	 */
	public final String user;
}
