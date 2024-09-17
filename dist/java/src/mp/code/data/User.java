package mp.code.data;

import lombok.EqualsAndHashCode;
import lombok.RequiredArgsConstructor;
import lombok.ToString;

import java.util.UUID;

/**
 * A data class holding information about a user.
 */
@ToString
@EqualsAndHashCode
@RequiredArgsConstructor
public class User {
	/**
	 * The {@link UUID} of the user.
	 */
	public final UUID id;

	/**
	 * The human-readable name of the user.
	 */
	public final String name;
}
