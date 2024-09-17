package mp.code.data;

import lombok.EqualsAndHashCode;
import lombok.RequiredArgsConstructor;
import lombok.ToString;

@ToString
@EqualsAndHashCode
@RequiredArgsConstructor
public class Cursor {
	public final int startRow, startCol, endRow, endCol;
	public final String buffer;
	public final String user;
}
