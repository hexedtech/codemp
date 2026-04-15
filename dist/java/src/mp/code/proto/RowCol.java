package mp.code.proto;

import lombok.EqualsAndHashCode;
import lombok.RequiredArgsConstructor;
import lombok.ToString;

/**
 * A data class representing a position in a buffer.
 */
@ToString
@EqualsAndHashCode
@RequiredArgsConstructor
public class RowCol {
	/**
	 * The row. If negative, it is clamped to 0.
	 */
	public final int row;

	/**
	 * The column. If negative, it is clamped to 0.
	 */
	public final int col;
}
