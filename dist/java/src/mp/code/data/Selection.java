package mp.code.data;

import lombok.EqualsAndHashCode;
import lombok.RequiredArgsConstructor;
import lombok.ToString;

/**
 * A data class holding information about a cursor selection.
 */
@ToString
@EqualsAndHashCode
@RequiredArgsConstructor
public class Selection {
	/**
	 * The starting row of the cursor position.
	 * If negative, it is clamped to 0.
	 */
	public final int startRow;

	/**
	 * The starting column of the cursor position.
	 * If negative, it is clamped to 0.
	 */
	public final int startCol;

	/**
	 * The ending row of the cursor position.
	 * If negative, it is clamped to 0.
	 */
	public final int endRow;

	/**
	 * The ending column of the cursor position.
	 * If negative, it is clamped to 0.
	 */
	public final int endCol;

	/**
	 * The buffer the cursor is located on.
	 */
	public final String buffer;
}
