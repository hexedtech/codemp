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
	type Error: std::fmt::Debug;

	/// Attempt to convert the given object to a [jni::objects::JObject].
	fn jobjectify(self, env: &mut jni::JNIEnv<'local>) -> Result<jni::objects::JObject<'local>, Self::Error>;
}

impl<'local> JObjectify<'local> for uuid::Uuid {
	type Error = jni::errors::Error;
	fn jobjectify(self, env: &mut jni::JNIEnv<'local>) -> Result<jni::objects::JObject<'local>, Self::Error> {
		env.find_class("java/util/UUID").and_then(|class| {
			let (msb, lsb) = self.as_u64_pair();
			let msb = i64::from_ne_bytes(msb.to_ne_bytes());
			let lsb = i64::from_ne_bytes(lsb.to_ne_bytes());
			env.new_object(&class, "(JJ)V", &[jni::objects::JValueGen::Long(msb), jni::objects::JValueGen::Long(lsb)])
		})
	}
}

impl<'local> JObjectify<'local> for crate::cursor::Controller {
	type Error = jni::errors::Error;

	fn jobjectify(self, env: &mut jni::JNIEnv<'local>) -> Result<jni::objects::JObject<'local>, Self::Error> {
		let class = env.find_class("mp/code/CursorController")?;
		env.new_object(
			class,
			"(J)V",
			&[
				jni::objects::JValueGen::Long(Box::into_raw(Box::new(&self)) as jni::sys::jlong)
			]
		)
	}
}

impl<'local> JObjectify<'local> for crate::buffer::Controller {
	type Error = jni::errors::Error;

	fn jobjectify(self, env: &mut jni::JNIEnv<'local>) -> Result<jni::objects::JObject<'local>, Self::Error> {
		let class = env.find_class("mp/code/BufferController")?;
		env.new_object(
			class,
			"(J)V",
			&[
				jni::objects::JValueGen::Long(Box::into_raw(Box::new(&self)) as jni::sys::jlong)
			]
		)
	}
}

macro_rules! handle_callback {
	($jtype:literal, $env:ident, $self_ptr:ident, $cb:ident, $t:ty) => {
		let controller = unsafe { Box::leak(Box::from_raw($self_ptr as *mut $t)) };
		
		let Ok(jvm) = $env.get_java_vm() else {
			$env.throw_new("mp/code/exceptions/JNIException", "Failed to get JVM reference!")
				.expect("Failed to throw exception!");
			return;
		};

		let Ok(cb_ref) = $env.new_global_ref($cb) else {
			$env.throw_new("mp/code/exceptions/JNIException", "Failed to pin callback reference!")
				.expect("Failed to throw exception!");
			return;
		};
		controller.callback(move |controller: $t| {
			use std::ops::DerefMut;
			use crate::ffi::java::JObjectify;
			let mut guard = jvm.attach_current_thread().unwrap();
			let jcontroller = match controller.jobjectify(guard.deref_mut()) {
				Err(e) => return tracing::error!("could not convert callback argument: {e:?}"),
				Ok(x) => x,
			};
			let sig = format!("(L{};)V", $jtype);
			if let Err(e) = guard.call_method(&cb_ref,
				"invoke",
				&sig,
				&[jni::objects::JValueGen::Object(&jcontroller)]
			) {
				tracing::error!("error invoking callback: {e:?}");
			}
		});
	};
}

pub(crate) use handle_callback;

