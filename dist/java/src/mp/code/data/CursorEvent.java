package mp.code.data;

import lombok.EqualsAndHashCode;
import lombok.RequiredArgsConstructor;
import lombok.ToString;

/**
 * A data class representing an event about a user's cursor.
 */
@ToString
@EqualsAndHashCode
@RequiredArgsConstructor
public class CursorEvent {
	/**
	 * The user who sent the cursor.
	 */
	public final String user;

	/**
	 * The cursor position data.
	 */
	public final Cursor cursor;
}
