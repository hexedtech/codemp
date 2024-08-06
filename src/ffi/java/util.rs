use jni::{JNIEnv, sys::jlong};

/// A simple utility method that converts a pointer back into a [Box] and then drops it.
pub(crate) fn dereference_and_drop<T>(ptr: jlong) {
	let client : Box<T> = unsafe { Box::from_raw(ptr as *mut T) };
	std::mem::drop(client)
}

/// A trait meant for our [crate::Result] type to make converting it to Java easier.
pub(crate) trait JExceptable<T> {
	/// Unwraps it and throws an appropriate Java exception if it's an error.
	/// Theoretically it returns the type's default value, but the exception makes the value ignored.
	fn jexcept(self, env: &mut JNIEnv) -> T;
}

impl<T> JExceptable<T> for crate::Result<T> where T: Default {
	fn jexcept(self, env: &mut JNIEnv) -> T {
		if let Err(err) = &self {
			let msg = format!("{err}");
			match err {
				crate::Error::InvalidState { .. } => env.throw_new("mp/code/exceptions/InvalidStateException", msg),
				crate::Error::Deadlocked => env.throw_new("mp/code/exceptions/DeadlockedException", msg),
				crate::Error::Transport { .. } => env.throw_new("mp/code/exceptions/TransportException", msg),
				crate::Error::Channel { .. } => env.throw_new("mp/code/exceptions/ChannelException", msg)
			}.expect("Failed to throw exception!");
		}
		self.unwrap_or_default()
	}
}
