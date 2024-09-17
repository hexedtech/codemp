package mp.code.data;

import lombok.AccessLevel;
import lombok.EqualsAndHashCode;
import lombok.RequiredArgsConstructor;
import lombok.ToString;

import java.util.Optional;
import java.util.OptionalInt;

/**
 * A data class representing the connection configuration.
 */
@ToString
@EqualsAndHashCode
@RequiredArgsConstructor(access = AccessLevel.PRIVATE)
@SuppressWarnings("OptionalUsedAsFieldOrParameterType")
public class Config {
	/** The username to connect with. */
	public final String username;
	/** The password to connect with. */
	public final String password;
	/** The host to connect to, if custom. */
	public final Optional<String> host;
	/** The port to connect to, if custom. */
	public final OptionalInt port;
	/** Whether to use TLS, if custom. */
	public final Optional<Boolean> tls;

	/**
	 * Provides the given username and password on the default server.
	 * @param username the username
	 * @param password the password
	 */
	public Config(String username, String password) {
		this(
			username,
			password,
			Optional.empty(),
			OptionalInt.empty(),
			Optional.empty()
		);
	}

	/**
	 * Provides the given username and password as well as a custom server.
	 * @param username the username
	 * @param password the password
	 * @param host the host server
	 * @param port the port CodeMP is running on, must be between 0 and 65535
	 * @param tls whether to use TLS
	 */
	public Config(String username, String password, String host, int port, boolean tls) {
		this(
			username,
			password,
			Optional.of(host),
			OptionalInt.of(checkPort(port)),
			Optional.of(tls)
		);
	}

	private static int checkPort(int port) {
		if(port < 0 || port > 65535)
			throw new IllegalArgumentException("Port value must be between 0 and 65535!");
		return port;
	}
}
