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
	 * The buffer the cursor is on.
	 */
	public final String buffer;

	/**
	 * The associated selection updates.
	 */
	public final Selection[] selection;
}
