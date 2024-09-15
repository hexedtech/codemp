package mp.code.data;

import java.util.UUID;

public class User {
	public final UUID id;
	public final String name;

	public User(UUID id, String name) {
		this.id = id;
		this.name = name;
	}
}
