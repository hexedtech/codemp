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

/// Utility macro that attempts to handle an error in a [Result].
/// MUST be called within a $result.is_err() block or similar. Failure to do so is UB.
/// Will return early with a provided return value, or panic if it fails to throw a Java exception.
macro_rules! handle_error {
	($env: expr, $result: ident, $return: expr) => {
		{
			let err = unsafe { $result.unwrap_err_unchecked() };
			tracing::info!("Attempting to throw error {err:#?} as a Java exception...");
			if let Err(e) = err.jobjectify($env).map(|t| t.into()).and_then(|t: jni::objects::JThrowable| $env.throw(&t)) {
				panic!("Failed to throw exception: {e}");
			}
			return $return;
		}
	};
}
pub(crate) use handle_error;

/// Performs a null check on the given variable and throws a NullPointerException on the Java side
/// if it is null. Finally, it returns with the given default value.
macro_rules! null_check {
	($env: ident, $var: ident, $return: expr) => {
		if $var.is_null() {
			let mut message = stringify!($var).to_string();
			message.push_str(" cannot be null!");
			$env.throw_new("java/lang/NullPointerException", message)
				.expect("Failed to throw exception!");
			return $return;
		}
	};
}
pub(crate) use null_check;


/// A trait meant for our local result type to make converting it to Java easier.
/// jni-rs technically has [jni::errors::ToException], but this approach keeps it stream-like.
pub(crate) trait JExceptable<'local, T: Default> {
	/// Unwrap it and throws an appropriate Java exception if it's an error.
	/// Theoretically it returns the type's default value, but the exception makes the value ignored.
	fn jexcept(self, env: &mut jni::JNIEnv<'local>) -> T;
}

impl<'local, T: Default, E: JObjectify<'local> + std::fmt::Debug> JExceptable<'local, T> for Result<T, E> {
	fn jexcept(self, env: &mut jni::JNIEnv<'local>) -> T {
		if let Ok(res) = self {
			res
		} else {
			handle_error!(env, self, Default::default());
		}
	}
}

/// Allows easy conversion for various types into Java objects.
/// This is similar to [TryInto], but for Java types.
pub(crate) trait JObjectify<'local> {
	/// Attempt to convert the given object to a [jni::objects::JObject].
	fn jobjectify(self, env: &mut jni::JNIEnv<'local>) -> Result<jni::objects::JObject<'local>, jni::errors::Error>;
}

macro_rules! jobjectify_error {
	($self: ident, $type: ty, $jclass: expr) => {
		impl<'local> JObjectify<'local> for $type {
			fn jobjectify($self, env: &mut jni::JNIEnv<'local>) -> Result<jni::objects::JObject<'local>, jni::errors::Error> {
				let class = env.find_class($jclass)?;
				let msg = env.new_string(format!("{:#?}", $self))?;
				env.new_object(class, "(Ljava/lang/String;)V", &[jni::objects::JValueGen::Object(&msg)])
			}
		}
	};
}

jobjectify_error!(self, crate::errors::RemoteError, "mp/code/exceptions/ConnectionRemoteException");
jobjectify_error!(self, jni::errors::Error, match self {
	jni::errors::Error::NullPtr(_) => "java/lang/NullPointerException",
	_ => "mp/code/exceptions/JNIException"
});
jobjectify_error!(self, uuid::Error, "java/lang/IllegalArgumentException");
jobjectify_error!(self, crate::errors::ConnectionError, match self { 
	crate::errors::ConnectionError::Transport(_) => "mp/code/exceptions/ConnectionTransportException",
	crate::errors::ConnectionError::Remote(_) => "mp/code/exceptions/ConnectionRemoteException"
});
jobjectify_error!(self, crate::errors::ControllerError, match self { 
	crate::errors::ControllerError::Stopped => "mp/code/exceptions/ControllerStoppedException",
	crate::errors::ControllerError::Unfulfilled => "mp/code/exceptions/ControllerUnfulfilledException",
});


impl<'local> JObjectify<'local> for uuid::Uuid {
	fn jobjectify(self, env: &mut jni::JNIEnv<'local>) -> Result<jni::objects::JObject<'local>, jni::errors::Error> {
		let class = env.find_class("java/util/UUID")?;
		let (msb, lsb) = self.as_u64_pair();
		let msb = i64::from_ne_bytes(msb.to_ne_bytes());
		let lsb = i64::from_ne_bytes(lsb.to_ne_bytes());
		env.new_object(&class, "(JJ)V", &[jni::objects::JValueGen::Long(msb), jni::objects::JValueGen::Long(lsb)])
	}
}

/// Generates a [JObjectify] implementation for a class that is just a holder for a pointer.
macro_rules! jobjectify_ptr_class {
	($type: ty, $jclass: literal) => {
		impl<'local> JObjectify<'local> for $type {
			fn jobjectify(self, env: &mut jni::JNIEnv<'local>) -> Result<jni::objects::JObject<'local>, jni::errors::Error> {
				let class = env.find_class($jclass)?;
				env.new_object(
					class,
					"(J)V",
					&[jni::objects::JValueGen::Long(Box::into_raw(Box::new(self)) as jni::sys::jlong)]
				)
			}
		}
	};
}

jobjectify_ptr_class!(crate::Client, "mp/code/Client");
jobjectify_ptr_class!(crate::Workspace, "mp/code/Workspace");
jobjectify_ptr_class!(crate::cursor::Controller, "mp/code/CursorController");
jobjectify_ptr_class!(crate::buffer::Controller, "mp/code/BufferController");

impl<'local> JObjectify<'local> for crate::api::User {
	fn jobjectify(self, env: &mut jni::JNIEnv<'local>) -> Result<jni::objects::JObject<'local>, jni::errors::Error> {
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

impl<'local> JObjectify<'local> for crate::api::Event {
	fn jobjectify(self, env: &mut jni::JNIEnv<'local>) -> Result<jni::objects::JObject<'local>, jni::errors::Error> {
		let (ordinal, arg) = match self {
			crate::api::Event::UserJoin(arg) => (0, env.new_string(arg)?),
			crate::api::Event::UserLeave(arg) => (1, env.new_string(arg)?),
			crate::api::Event::FileTreeUpdated(arg) => (2, env.new_string(arg)?),
		};

		let type_class = env.find_class("mp/code/Workspace$Event$Type")?;
		let variants: jni::objects::JObjectArray = env.call_method(
			type_class,
			"getEnumConstants",
			"()[Ljava/lang/Object;",
			&[]
		)?.l()?.into();
		let event_type = env.get_object_array_element(variants, ordinal)?;
		
		let event_class = env.find_class("mp/code/Workspace$Event")?;
		env.new_object(
			event_class,
			"(Lmp/code/Workspace$Event$Type;Ljava/lang/String;)V",
			&[
				jni::objects::JValueGen::Object(&event_type),
				jni::objects::JValueGen::Object(&arg)
			]
		)
	}
}

impl<'local> JObjectify<'local> for crate::workspace::DetachResult {
	fn jobjectify(self, env: &mut jni::JNIEnv<'local>) -> Result<jni::objects::JObject<'local>, jni::errors::Error> {
		let ordinal = match self {
			crate::workspace::DetachResult::NotAttached => 0,
			crate::workspace::DetachResult::Detaching => 1,
			crate::workspace::DetachResult::AlreadyDetached => 2
		};

		let class = env.find_class("mp/code/data/DetachResult")?;
		let variants: jni::objects::JObjectArray = env.call_method(
			class,
			"getEnumConstants",
			"()[Ljava/lang/Object;",
			&[]
		)?.l()?.into();
		env.get_object_array_element(variants, ordinal)
	}
}

impl<'local> JObjectify<'local> for crate::api::TextChange {
	fn jobjectify(self, env: &mut jni::JNIEnv<'local>) -> Result<jni::objects::JObject<'local>, jni::errors::Error> {
		let content = env.new_string(self.content)?;

		let hash = env.find_class("java/util/OptionalLong").and_then(|class| {
			if let Some(h) = self.hash {
				env.call_static_method(class, "of", "(J)Ljava/util/OptionalLong;", &[jni::objects::JValueGen::Long(h)])
			} else {
				env.call_static_method(class, "empty", "()Ljava/util/OptionalLong;", &[])
			}
		}).and_then(|o| o.l())?;
		env.find_class("mp/code/data/TextChange").and_then(|class| {
			env.new_object(
				class,
				"(JJLjava/lang/String;Ljava/util/OptionalLong;)V",
				&[
					jni::objects::JValueGen::Long(self.start.into()),
					jni::objects::JValueGen::Long(self.end.into()),
					jni::objects::JValueGen::Object(&content),
					jni::objects::JValueGen::Object(&hash)
				]
			)
		})
	}
}

impl<'local> JObjectify<'local> for crate::api::Cursor {
	fn jobjectify(self, env: &mut jni::JNIEnv<'local>) -> Result<jni::objects::JObject<'local>, jni::errors::Error> {
		env.find_class("mp/code/data/Cursor").and_then(|class| {
			let buffer = env.new_string(&self.buffer)?;
			let user = if let Some(user) = self.user {
				env.new_string(user)?.into()
			} else {
				jni::objects::JObject::null()
			};

			env.new_object(
				class,
				"(IIIILjava/lang/String;Ljava/lang/String;)V",
				&[
					jni::objects::JValueGen::Int(self.start.0),
					jni::objects::JValueGen::Int(self.start.1),
					jni::objects::JValueGen::Int(self.end.0),
					jni::objects::JValueGen::Int(self.end.1),
					jni::objects::JValueGen::Object(&buffer),
					jni::objects::JValueGen::Object(&user)
				]
			)
		})
	}
}

/// Allows easy conversion of Java types into their Rust counterparts.
pub(crate) trait Deobjectify<'local, T: Sized> {
	/// Attempt to convert the given [jni::objects::JObject] into its Rust counterpart.
	fn deobjectify(env: &mut jni::JNIEnv<'local>, jobject: jni::objects::JObject<'local>) -> Result<T, jni::errors::Error>;
}

impl<'local> Deobjectify<'local, Self> for crate::api::Config {
	fn deobjectify(env: &mut jni::JNIEnv<'local>, config: jni::objects::JObject<'local>) -> Result<Self, jni::errors::Error> {
		let username = {
			let jfield = env.get_field(&config, "username", "Ljava/lang/String;")?.l()?;
			if jfield.is_null() {
				return Err(jni::errors::Error::NullPtr("Username can never be null!"));
			}
			unsafe { env.get_string_unchecked(&jfield.into()) }?.into()
		};

		let password = {
			let jfield = env.get_field(&config, "password", "Ljava/lang/String;")?.l()?;
			if jfield.is_null() {
				return Err(jni::errors::Error::NullPtr("Password can never be null!"));
			}
			unsafe { env.get_string_unchecked(&jfield.into()) }?.into()
		};

		let host = {
			let jfield = env.get_field(&config, "host", "Ljava/util/Optional;")?.l()?;
			if env.call_method(&jfield, "isPresent", "()Z", &[])?.z()? {
				let field = env.call_method(&jfield, "get", "()Ljava/lang/Object;", &[])?.l()?;
				Some(unsafe { env.get_string_unchecked(&field.into()) }?.into())
			} else {
				None
			}
		};

		let port = {
			let jfield = env.get_field(&config, "port", "Ljava/util/OptionalInt;")?.l()?;
			if env.call_method(&jfield, "isPresent", "()Z", &[])?.z()? {
				let ivalue = env.call_method(&jfield, "getAsInt", "()I", &[])?.i()?;
				Some(ivalue.clamp(0, 65535) as u16)
			} else {
				None
			}
		};

		let tls = {
			let jfield = env.get_field(&config, "host", "Ljava/util/Optional;")?.l()?;
			if env.call_method(&jfield, "isPresent", "()Z", &[])?.z()? {
				let field = env.call_method(&jfield, "get", "()Ljava/lang/Object;", &[])?.l()?;
				let bool_true = env.get_static_field("java/lang/Boolean", "TRUE", "Ljava/lang/Boolean;")?.l()?;
				Some(env.call_method(
					field,
					"equals",
					"(Ljava/lang/Object;)Z",
					&[jni::objects::JValueGen::Object(&bool_true)]
				)?.z()?) // what a joke
			} else {
				None
			}
		};

		Ok(Self { username, password, host, port, tls })
	}
}

impl<'local> Deobjectify<'local, Self> for crate::api::Cursor {
	fn deobjectify(env: &mut jni::JNIEnv<'local>, cursor: jni::objects::JObject<'local>) -> Result<Self, jni::errors::Error> {
		let start_row = env.get_field(&cursor, "startRow", "I")?.i()?;
		let start_col = env.get_field(&cursor, "startCol", "I")?.i()?;
		let end_row = env.get_field(&cursor, "endRow", "I")?.i()?;
		let end_col = env.get_field(&cursor, "endCol", "I")?.i()?;

		let buffer = {
			let jfield = env.get_field(&cursor, "buffer", "Ljava/lang/String;")?.l()?;
			if jfield.is_null() {
				return Err(jni::errors::Error::NullPtr("Buffer can never be null!"));
			}
			unsafe { env.get_string_unchecked(&jfield.into()) }?.into()
		};
	
		let user = {
			let jfield = env.get_field(&cursor, "user", "Ljava/lang/String;")?.l()?;
			if jfield.is_null() {
				None
			} else {
				Some(unsafe { env.get_string_unchecked(&jfield.into()) }?.into())
			}
		};

		Ok(Self { start: (start_row, start_col), end: (end_row, end_col), buffer, user })
	}
}

impl<'local> Deobjectify<'local, Self> for crate::api::TextChange {
	fn deobjectify(env: &mut jni::JNIEnv<'local>, change: jni::objects::JObject<'local>) -> Result<Self, jni::errors::Error> {
		let start = env.get_field(&change, "start", "J")?.j()?.clamp(0, u32::MAX.into()) as u32;
		let end = env.get_field(&change, "end", "J")?.j()?.clamp(0, u32::MAX.into()) as u32;

		let content = {
			let jfield = env.get_field(&change, "content", "Ljava/lang/String;")?.l()?;
			if jfield.is_null() {
				return Err(jni::errors::Error::NullPtr("Content can never be null!"));
			}
			unsafe { env.get_string_unchecked(&jfield.into()) }?.into()
		};

		let hash = {
			let jfield = env.get_field(&change, "hash", "Ljava/util/OptionalLong;")?.l()?;
			if env.call_method(&jfield, "isPresent", "()Z", &[])?.z()? {
				Some(env.call_method(&jfield, "getAsLong", "()J", &[])?.j()?)
			} else {
				None
			}
		};
		Ok(Self { start, end, content, hash })
	}
}
