package mp.code.data;

import java.util.OptionalLong;

public class TextChange {
	public final long start;
	public final long end;
	public final String content;
	public final OptionalLong hash; // xxh3 hash

	public TextChange(long start, long end, String content, OptionalLong hash) {
		this.start = start;
		this.end = end;
		this.content = content;
		this.hash = hash;
	}
}
