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
public class TextChange {
	/**
	 * The starting position of the change.
	 * If negative, it is clamped to 0.
	 */
	public final long startIdx;

	/**
	 * The ending position of the change.
	 * If negative, it is clamped to 0.
	 */
	public final long endIdx;

	/**
	 * The content of the change.
	 * It should never be null; if you need to represent absence of content, use an empty string.
	 */
	public final String content;

	/**
	 * Checks if the change represents a deletion.
	 * It does if the starting index is lower than the ending index.
	 * It is NOT mutually exclusive with {@link #isInsert()}.
	 * @return true if this change represents a deletion
	 */
	public boolean isDelete() {
		return this.startIdx < this.endIdx;
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
		long preIndex = Math.min(this.startIdx, input.length());
		String pre = "";
		try {
			pre = input.substring(0, (int) preIndex);
		} catch(IndexOutOfBoundsException ignored) {}
		String post = "";
		try {
			post = input.substring((int) this.endIdx);
		} catch(IndexOutOfBoundsException ignored) {}
		return pre + this.content + post;
	}
}
