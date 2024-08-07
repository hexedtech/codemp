package mp.code.data;

public class Cursor {
	public final int startRow, startCol, endRow, endCol;
	public final String buffer;
	public final String user;

	public Cursor(int startRow, int startCol, int endRow, int endCol, String buffer, String user) {
		this.startRow = startRow;
		this.startCol = startCol;
		this.endRow = endRow;
		this.endCol = endCol;
		this.buffer = buffer;
		this.user = user;
	}
}
