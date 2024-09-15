package mp.code.data;

@FunctionalInterface
public interface Callback<T> {
	void invoke(T controller);
}
