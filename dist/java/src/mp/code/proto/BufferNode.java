package mp.code.proto;

import lombok.EqualsAndHashCode;
import lombok.RequiredArgsConstructor;
import lombok.ToString;

/**
 * The unique identifier of a buffer.
 */
@ToString
@EqualsAndHashCode
@RequiredArgsConstructor
public class BufferNode {
	/**
	 * The path the buffer is treated to be in, UNIX-type.
	 */
	public final String path;

	/**
	 * The attributes of this buffer.
	 */
	public final BufferAttributes attributes;
}
