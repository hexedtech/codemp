package mp.code.data;

public class TextChange {
	public final long start;
	public final long end;
	public final String content;
	private final long hash; // xxh3 hash

	public TextChange(long start, long end, String content, long hash) {
		this.start = start;
		this.end = end;
		this.content = content;
		this.hash = hash;
	}

	private static native long hash(String content);
	public boolean hashMatches(String content) {
		// 0 is Rust default value and a very unlikely hash
		return hash == 0L || this.hash == hash(content);
	}
}
