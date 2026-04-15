package mp.code.proto;

import lombok.EqualsAndHashCode;
import lombok.RequiredArgsConstructor;
import lombok.ToString;

/**
 * The attributes of a buffer.
 */
@ToString
@EqualsAndHashCode
@RequiredArgsConstructor
public class BufferAttributes {
	/**
	 * Whether this buffer gets auto-deleted once all users leave.
	 */
	public final boolean ephemeral;
}
