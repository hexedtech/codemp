package mp.code.data;

import lombok.AccessLevel;
import lombok.EqualsAndHashCode;
import lombok.RequiredArgsConstructor;
import lombok.ToString;

import java.util.Optional;
import java.util.OptionalInt;

@ToString
@EqualsAndHashCode
@RequiredArgsConstructor(access = AccessLevel.PRIVATE)
@SuppressWarnings("OptionalUsedAsFieldOrParameterType")
public class Config {
	private final String username;
	private final String password;
	private final Optional<String> host;
	private final OptionalInt port;
	private final Optional<Boolean> tls;

	public Config(String username, String password) {
		this(
			username,
			password,
			Optional.empty(),
			OptionalInt.empty(),
			Optional.empty()
		);
	}

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
