package mp.code.data;

import lombok.EqualsAndHashCode;
import lombok.RequiredArgsConstructor;
import lombok.ToString;

import java.util.UUID;

@ToString
@EqualsAndHashCode
@RequiredArgsConstructor
public class User {
	public final UUID id;
	public final String name;
}
