package mp.code.data;

public class TextChange {
	public final long start;
	public final long end;
	public final String content;

	public TextChange(long start, long end, String content) {
		this.start = start;
		this.end = end;
		this.content = content;
	}
}
