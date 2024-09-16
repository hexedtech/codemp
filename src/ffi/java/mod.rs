pub mod client;
pub mod workspace;
pub mod cursor;
pub mod buffer;
pub mod ext;

/// Gets or creates the relevant [tokio::runtime::Runtime].
fn tokio() -> &'static tokio::runtime::Runtime {
	use std::sync::OnceLock;
	static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
	RT.get_or_init(||
		tokio::runtime::Builder::new_current_thread()
			.enable_all()
			.build()
			.expect("could not create tokio runtime")
	)
}

/// A static reference to [jni::JavaVM] that is set on JNI load.
static mut JVM: Option<std::sync::Arc<jni::JavaVM>> = None;

/// Safe accessor for the [jni::JavaVM] static.
pub(crate) fn jvm() -> std::sync::Arc<jni::JavaVM> {
	unsafe { JVM.clone() }.unwrap()
}

/// Called upon initialisation of the JVM.
#[allow(non_snake_case)]
#[no_mangle]
pub extern "system" fn JNI_OnLoad(
	vm: jni::JavaVM,
	_: *mut std::ffi::c_void
) -> jni::sys::jint {
	unsafe { JVM = Some(std::sync::Arc::new(vm)) };
	jni::sys::JNI_VERSION_1_1
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
			if let Err(err) = env.throw_new("mp/code/exceptions/JNIException", msg) {
				if let Err(err) = env.exception_describe() {
					tracing::error!("An exception occurred and we failed to even describe it: {err:#?}.");
				}
				panic!("A severe error occurred: we were unable to create a JNIException from {err:#?}. This is an unrecoverable state.");
			}
		}
		self.unwrap_or_default()
	}
}

impl<T> JExceptable<T> for Result<T, uuid::Error> where T: Default {
	fn jexcept(self, env: &mut jni::JNIEnv) -> T {
		if let Err(err) = &self {
			let msg = format!("{err}");
			if let Err(err) = env.throw_new("java/lang/IllegalArgumentException", msg) {
				if let Err(err) = env.exception_describe() {
					tracing::error!("An exception occurred and we failed to even describe it: {err:#?}.");
				}
				panic!("A severe error occurred: we were unable to create a JNIException from {err:#?}. This is an unrecoverable state.");
			}
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
		let class = env.find_class("java/util/UUID")?;
		let (msb, lsb) = self.as_u64_pair();
		let msb = i64::from_ne_bytes(msb.to_ne_bytes());
		let lsb = i64::from_ne_bytes(lsb.to_ne_bytes());
		env.new_object(&class, "(JJ)V", &[jni::objects::JValueGen::Long(msb), jni::objects::JValueGen::Long(lsb)])
	}
}

impl<'local> JObjectify<'local> for crate::api::User {
	type Error = jni::errors::Error;

	fn jobjectify(self, env: &mut jni::JNIEnv<'local>) -> Result<jni::objects::JObject<'local>, Self::Error> {
		let id_field = self.id.jobjectify(env)?;
		let name_field = env.new_string(self.name)?;
		let class = env.find_class("mp/code/data/User")?;
		env.new_object(
			&class,
			"(Ljava/util/UUID;Ljava/lang/String;)V",
			&[
				jni::objects::JValueGen::Object(&id_field),
				jni::objects::JValueGen::Object(&name_field)
			]
		)
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
				jni::objects::JValueGen::Long(Box::into_raw(Box::new(self)) as jni::sys::jlong)
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
				jni::objects::JValueGen::Long(Box::into_raw(Box::new(self)) as jni::sys::jlong)
			]
		)
	}
}
