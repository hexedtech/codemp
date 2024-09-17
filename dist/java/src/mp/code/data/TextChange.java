package mp.code.data;

import lombok.EqualsAndHashCode;
import lombok.RequiredArgsConstructor;
import lombok.ToString;

import java.util.OptionalLong;

@ToString
@EqualsAndHashCode
@RequiredArgsConstructor
@SuppressWarnings("OptionalUsedAsFieldOrParameterType")
public class TextChange {
	public final int start;
	public final int end;
	public final String content;
	public final OptionalLong hash; // xxh3 hash

	public boolean isDelete() {
		return this.start != this.end;
	}

	public boolean isInsert() {
		return !this.content.isEmpty();
	}

	public boolean isEmpty() {
		return !this.isDelete() && !this.isInsert();
	}

	//TODO: apply()
}
