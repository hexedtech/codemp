pub mod client;
pub mod workspace;
pub mod cursor;
pub mod buffer;

lazy_static::lazy_static! {
	pub(crate) static ref RT: tokio::runtime::Runtime = tokio::runtime::Runtime::new().expect("could not create tokio runtime");
}

/// Sets up logging. Useful for debugging.
pub(crate) fn setup_logger(debug: bool, path: Option<String>) {
	let format = tracing_subscriber::fmt::format()
		.with_level(true)
		.with_target(true)
		.with_thread_ids(false)
		.with_thread_names(false)
		.with_ansi(false)
		.with_file(false)
		.with_line_number(false)
		.with_source_location(false)
		.compact();

	let level = if debug { tracing::Level::DEBUG } else {tracing::Level::INFO };

	let builder = tracing_subscriber::fmt()
		.event_format(format)
		.with_max_level(level);

	if let Some(path) = path {
		let logfile = std::fs::File::create(path).expect("failed creating logfile");
		builder.with_writer(std::sync::Mutex::new(logfile)).init();
	} else {
		builder.with_writer(std::sync::Mutex::new(std::io::stdout())).init();
	}
}

/// A trait meant for our [crate::Result] type to make converting it to Java easier.
/// jni-rs technically has [jni::errors::ToException], but this approach keeps it stream-like.
pub(crate) trait JExceptable<T> {
	/// Unwraps it and throws an appropriate Java exception if it's an error.
	/// Theoretically it returns the type's default value, but the exception makes the value ignored.
	fn jexcept(self, env: &mut jni::JNIEnv) -> T;
}

impl<T> JExceptable<T> for crate::Result<T> where T: Default {
	fn jexcept(self, env: &mut jni::JNIEnv) -> T {
		if let Err(err) = &self {
			let msg = format!("{err}");
			match err {
				crate::Error::InvalidState { .. } => env.throw_new("mp/code/exceptions/InvalidStateException", msg),
				crate::Error::Deadlocked => env.throw_new("mp/code/exceptions/DeadlockedException", msg),
				crate::Error::Transport { .. } => env.throw_new("mp/code/exceptions/TransportException", msg),
				crate::Error::Channel { .. } => env.throw_new("mp/code/exceptions/ChannelException", msg)
			}.jexcept(env);
		}
		self.unwrap_or_default()
	}
}

impl<T> JExceptable<T> for Result<T, jni::errors::Error> where T: Default {
	fn jexcept(self, env: &mut jni::JNIEnv) -> T {
		if let Err(err) = &self {
			let msg = format!("{err}");
			env.throw_new("mp/code/exceptions/JNIException", msg)
				.expect("A severe error occurred: we were unable to create a JNIException. This is an unrecoverable state.");
		}
		self.unwrap_or_default()
	}
}

impl<T> JExceptable<T> for Result<T, uuid::Error> where T: Default {
	fn jexcept(self, env: &mut jni::JNIEnv) -> T {
		if let Err(err) = &self {
			let msg = format!("{err}");
			env.throw_new("java/lang/IllegalArgumentException", msg) 
				.expect("A severe error occurred: we were unable to create a JNIException. This is an unrecoverable state.");
		}
		self.unwrap_or_default()
	}
}
