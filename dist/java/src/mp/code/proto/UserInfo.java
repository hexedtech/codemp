package mp.code.proto;

import lombok.EqualsAndHashCode;
import lombok.RequiredArgsConstructor;
import lombok.ToString;

/**
 * A data class holding information about a user.
 */
@ToString
@EqualsAndHashCode
@RequiredArgsConstructor
public class UserInfo {
	/**
	 * The unique name of the user.
	 */
	public final String name;

	/**
	 * The visible name of the user, may be null.
	 */
	public final String displayName;

	/**
	 * User description ("bio").
	 */
	public final String description;

	/**
	 * A small image some editors can display, may be null.
	 */
	public final byte[] avatar;
}
