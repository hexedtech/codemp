package mp.code.proto;

import lombok.EqualsAndHashCode;
import lombok.RequiredArgsConstructor;
import lombok.ToString;

/**
 * A data class holding information about a cursor event.
 */
@ToString
@EqualsAndHashCode
@RequiredArgsConstructor
public class CursorUpdate {
	/**
	 * The buffer the cursor is on.
	 */
	public final String buffer;

	/**
	 * The associated selection updates.
	 */
	public final CursorPosition[] cursors;
}
