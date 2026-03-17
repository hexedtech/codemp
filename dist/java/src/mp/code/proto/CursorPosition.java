package mp.code.proto;

import lombok.EqualsAndHashCode;
import lombok.RequiredArgsConstructor;
import lombok.ToString;

/**
 * A data class holding information about a cursor selection.
 */
@ToString
@EqualsAndHashCode
@RequiredArgsConstructor
public class CursorPosition {
	/**
	 * The start of the cursor's position.
	 */
	public final RowCol start;

	/**
	 * The end of the cursor's position.
	 */
	public final RowCol end;
}
