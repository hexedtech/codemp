package mp.code.data;

import lombok.EqualsAndHashCode;
import lombok.RequiredArgsConstructor;
import lombok.ToString;

/**
 * A data class holding information about a cursor event.
 */
@ToString
@EqualsAndHashCode
@RequiredArgsConstructor
public class Cursor {
	/**
	 * The user who controls the cursor.
	 */
	public final String user;

	/**
	 * The associated selection update.
	 */
	public final Selection selection;
}
