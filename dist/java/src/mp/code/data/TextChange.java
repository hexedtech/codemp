package mp.code.data;

import lombok.EqualsAndHashCode;
import lombok.RequiredArgsConstructor;
import lombok.ToString;
import mp.code.Extensions;

import java.util.OptionalLong;

/**
 * A data class holding information about a text change.
 */
@ToString
@EqualsAndHashCode
@RequiredArgsConstructor
@SuppressWarnings("OptionalUsedAsFieldOrParameterType")
public class TextChange {
	/**
	 * The starting position of the change.
	 * If negative, it is clamped to 0.
	 */
	public final int start;

	/**
	 * The endomg position of the change.
	 * If negative, it is clamped to 0.
	 */
	public final int end;

	/**
	 * The content of the change.
	 * It should never be null; if you need to represent absence of content, use an empty string.
	 */
	public final String content;

	/**
	 * The hash of the content after applying it (calculated with {@link Extensions#hash(String)}).
	 * It is generally meaningless to send, but when received it is an invitation to check the hash
	 * and forcefully re-sync if necessary.
	 */
	public final OptionalLong hash; // xxh3 hash

	/**
	 * Checks if the change represents a deletion.
	 * It does if the starting index is lower than the ending index.
	 * It is NOT mutually exclusive with {@link #isInsert()}.
	 * @return true if this change represents a deletion
	 */
	public boolean isDelete() {
		return this.start < this.end;
	}

	/**
	 * Checks if the change represents an insertion.
	 * It does if the content is not empty
	 * It is NOT mutually exclusive with {@link #isDelete()}.
	 * @return true if this change represents an insertion
	 */
	public boolean isInsert() {
		return !this.content.isEmpty();
	}

	/**
	 * Checks whether this change is a no-op.
	 * @return true if this change is a no-op
	 */
	public boolean isEmpty() {
		return !this.isDelete() && !this.isInsert();
	}

	/**
	 * Applies the change to an input string and returns the result.
	 * @param input the input string
	 * @return the mutated string
	 */
	public String apply(String input) {
		int preIndex = Math.min(this.start, input.length());
		String pre = "";
		try {
			pre = input.substring(0, preIndex);
		} catch(IndexOutOfBoundsException ignored) {}
		String post = "";
		try {
			post = input.substring(this.end);
		} catch(IndexOutOfBoundsException ignored) {}
		return pre + this.content + post;
	}
}
