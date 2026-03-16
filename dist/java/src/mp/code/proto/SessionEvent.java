package mp.code.proto;

import lombok.EqualsAndHashCode;
import lombok.RequiredArgsConstructor;
import lombok.ToString;

@ToString
@EqualsAndHashCode
@RequiredArgsConstructor
public class SessionEvent {
	/**
	 * The underlying type of event, which will determine which fields are available.
	 * Always one of the constants from {@link SessionEventKind}.
	 */
	public final int kind;

	/**
	 * The user that joined or left, or null.
	 */
	public final String user;

	/**
	 * The {@link WorkspaceIdentifier} of the relevant workspace.
	 */
	public final WorkspaceIdentifier workspace;
}
