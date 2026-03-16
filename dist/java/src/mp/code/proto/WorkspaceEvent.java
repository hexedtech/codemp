package mp.code.proto;

import lombok.EqualsAndHashCode;
import lombok.RequiredArgsConstructor;
import lombok.ToString;

@ToString
@EqualsAndHashCode
@RequiredArgsConstructor
public class WorkspaceEvent {
	/**
	 * The underlying type of event, which will determine which fields are available.
	 * Always one of the constants from {@link WorkspaceEventKind}.
	 */
	public final int kind;

	/**
	 * The user that joined or left, or null.
	 */
	public final String user;

	/**
	 * The path of the relevant buffer, or null.
	 */
	public final String path;

	/**
	 * Whether the buffer is ephemeral, or null.
	 */
	public final Boolean ephemeral;

	/**
	 * The new path of the buffer after the rename, or null.
	 */
	public final String after;
}
