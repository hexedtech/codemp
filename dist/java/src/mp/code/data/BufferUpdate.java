package mp.code.data;

import lombok.EqualsAndHashCode;
import lombok.RequiredArgsConstructor;
import lombok.ToString;
import mp.code.Extensions;

import java.util.OptionalLong;

/**
 * A data class holding information about a buffer update.
 */
@ToString
@EqualsAndHashCode
@RequiredArgsConstructor
@SuppressWarnings("OptionalUsedAsFieldOrParameterType")
public class BufferUpdate {
	/**
	 * The hash of the content after applying it (calculated with {@link Extensions#hash(String)}).
	 * It is generally meaningless to send, but when received it is an invitation to check the hash
	 * and forcefully re-sync if necessary.
	 */
	public final OptionalLong hash; // xxh3 hash

	/**
	 * The CRDT version after the associated change has been applied.
	 * You MUST acknowledge that it was applied with {@link mp.code.BufferController#ack(long[])}.
	 */
	public final long[] version;

	/**
	 * The {@link TextChange} contained in this buffer update.
	 */
	public final TextChange change;
}
