package mp.code.proto;

import lombok.AccessLevel;
import lombok.EqualsAndHashCode;
import lombok.RequiredArgsConstructor;
import lombok.ToString;

/**
 * A data class representing the connection configuration.
 */
@ToString
@EqualsAndHashCode
@RequiredArgsConstructor(access = AccessLevel.PRIVATE)
public class Config {
	/** The username to connect with. */
	public final String username;
	/** The password to connect with. */
	public final String password;
	/** The host to connect to, if custom. Null otherwise. */
	public final String host;
	/** The port to connect to, if custom. Null otherwise. */
	public final Integer port;
	/** Whether to use TLS, if custom. Null otherwise. */
	public final Boolean tls;

	/**
	 * Provides the given username and password on the default server.
	 * @param username the username
	 * @param password the password
	 */
	public Config(String username, String password) {
		this(username, password, null, null, null);
	}
}
