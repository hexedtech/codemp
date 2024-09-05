//! ### java
//! Since for java it is necessary to deal with the JNI and no complete FFI library is available,
//! java glue directly writes JNI functions leveraging [jni] rust bindings.
//!
//! To have a runnable `jar`, some extra Java code must be compiled (available under `dist/java`)
//! and bundled together with the shared object. Such extra wrapper provides classes and methods
//! loading the native extension and invoking the underlying native functions.

pub mod client;
pub mod workspace;
pub mod cursor;
pub mod buffer;
pub mod utils;

lazy_static::lazy_static! {
	pub(crate) static ref RT: tokio::runtime::Runtime = tokio::runtime::Runtime::new().expect("could not create tokio runtime");
}

/// Set up logging. Useful for debugging.
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

/// A trait meant for our local result type to make converting it to Java easier.
/// jni-rs technically has [jni::errors::ToException], but this approach keeps it stream-like.
pub(crate) trait JExceptable<T> {
	/// Unwrap it and throws an appropriate Java exception if it's an error.
	/// Theoretically it returns the type's default value, but the exception makes the value ignored.
	fn jexcept(self, env: &mut jni::JNIEnv) -> T;
}

impl<T> JExceptable<T> for crate::errors::ConnectionResult<T> where T: Default {
	fn jexcept(self, env: &mut jni::JNIEnv) -> T {
		if let Err(err) = &self {
			let msg = format!("{err}");
			match err {
				crate::errors::ConnectionError::Transport(_) => env.throw_new("mp/code/exceptions/ConnectionTransportException", msg),
				crate::errors::ConnectionError::Remote(_) => env.throw_new("mp/code/exceptions/ConnectionRemoteException", msg),
			}.jexcept(env);
		}
		self.unwrap_or_default()
	}
}

impl<T> JExceptable<T> for crate::errors::RemoteResult<T> where T: Default {
	fn jexcept(self, env: &mut jni::JNIEnv) -> T {
		if let Err(err) = &self {
			let msg = format!("{err}");
			env.throw_new("mp/code/exceptions/connection/RemoteException", msg).jexcept(env);
		}
		self.unwrap_or_default()
	}
}

impl<T> JExceptable<T> for crate::errors::ControllerResult<T> where T: Default {
	fn jexcept(self, env: &mut jni::JNIEnv) -> T {
		if let Err(err) = &self {
			let msg = format!("{err}");
			match err {
				crate::errors::ControllerError::Stopped => env.throw_new("mp/code/exceptions/ControllerStoppedException", msg),
				crate::errors::ControllerError::Unfulfilled => env.throw_new("mp/code/exceptions/ControllerUnfulfilledException", msg),
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

/// Allows easy conversion for various types into Java objects.
/// This is essentially the same as [TryInto], but that can't be emplemented on non-local types.
pub(crate) trait JObjectify<'local> {
	/// The error type, likely to be [jni::errors::Error].
	type Error;

	/// Attempt to convert the given object to a [jni::objects::JObject].
	fn jobjectify(self, env: &mut jni::JNIEnv<'local>) -> Result<jni::objects::JObject<'local>, Self::Error>;
}

impl<'local> JObjectify<'local> for uuid::Uuid {
	type Error = jni::errors::Error;
	fn jobjectify(self, env: &mut jni::JNIEnv<'local>) -> Result<jni::objects::JObject<'local>, jni::errors::Error> {
		env.find_class("java/util/UUID").and_then(|class| {
			let (msb, lsb) = self.as_u64_pair();
			let msb = i64::from_ne_bytes(msb.to_ne_bytes());
			let lsb = i64::from_ne_bytes(lsb.to_ne_bytes());
			env.new_object(&class, "(JJ)V", &[jni::objects::JValueGen::Long(msb), jni::objects::JValueGen::Long(lsb)])
		})
	}
}
