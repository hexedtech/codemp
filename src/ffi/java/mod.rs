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

/// Performs a null check on the given variable and throws a NullPointerException on the Java side
/// if it is null. Finally, it returns with the given default value.
macro_rules! null_check { // TODO replace
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

impl jni_toolbox::JniToolboxError for crate::errors::ConnectionError {
	fn jclass(&self) -> String {
		match self { 
			crate::errors::ConnectionError::Transport(_) => "mp/code/exceptions/ConnectionTransportException",
			crate::errors::ConnectionError::Remote(_) => "mp/code/exceptions/ConnectionRemoteException"
		}.to_string()
	}
}

impl jni_toolbox::JniToolboxError for crate::errors::RemoteError {
	fn jclass(&self) -> String {
		"mp/code/exceptions/ConnectionRemoteException".to_string()
	}
}

impl jni_toolbox::JniToolboxError for crate::errors::ControllerError {
	fn jclass(&self) -> String {
		match self { 
			crate::errors::ControllerError::Stopped => "mp/code/exceptions/ControllerStoppedException",
			crate::errors::ControllerError::Unfulfilled => "mp/code/exceptions/ControllerUnfulfilledException",
		}.to_string()
	}
}

/// Generates a [JObjectify] implementation for a class that is just a holder for a pointer.
macro_rules! into_java_ptr_class {
	($type: ty, $jclass: literal) => {
		impl<'j> jni_toolbox::IntoJavaObject<'j> for $type {
			const CLASS: &'static str = $jclass;
			fn into_java_object(self, env: &mut jni::JNIEnv<'j>) -> Result<jni::objects::JObject<'j>, jni::errors::Error> {
				let class = env.find_class(Self::CLASS)?;
				env.new_object(
					class,
					"(J)V",
					&[jni::objects::JValueGen::Long(Box::into_raw(Box::new(self)) as jni::sys::jlong)]
				)
			}
		}
	};
}

into_java_ptr_class!(crate::Client, "mp/code/Client");
into_java_ptr_class!(crate::Workspace, "mp/code/Workspace");
into_java_ptr_class!(crate::cursor::Controller, "mp/code/CursorController");
into_java_ptr_class!(crate::buffer::Controller, "mp/code/BufferController");

impl<'j> jni_toolbox::IntoJavaObject<'j> for crate::api::User {
	const CLASS: &'static str = "mp/code/data/User";
	fn into_java_object(self, env: &mut jni::JNIEnv<'j>) -> Result<jni::objects::JObject<'j>, jni::errors::Error> {
		let id_field = self.id.into_java_object(env)?;
		let name_field = env.new_string(self.name)?;
		let class = env.find_class(Self::CLASS)?;
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

impl<'j> jni_toolbox::IntoJavaObject<'j> for crate::api::Event {
	const CLASS: &'static str = "mp/code/Workspace$Event";
	fn into_java_object(self, env: &mut jni::JNIEnv<'j>) -> Result<jni::objects::JObject<'j>, jni::errors::Error> { 
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
		
		let event_class = env.find_class(Self::CLASS)?;
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

impl<'j> jni_toolbox::IntoJavaObject<'j> for crate::api::TextChange {
	const CLASS: &'static str = "mp/code/data/TextChange";
	fn into_java_object(self, env: &mut jni::JNIEnv<'j>) -> Result<jni::objects::JObject<'j>, jni::errors::Error> {
		let content = env.new_string(self.content)?;

		let hash_class = env.find_class("java/util/OptionalLong")?;
		let hash = if let Some(h) = self.hash {
			env.call_static_method(hash_class, "of", "(J)Ljava/util/OptionalLong;", &[jni::objects::JValueGen::Long(h)])
		} else {
			env.call_static_method(hash_class, "empty", "()Ljava/util/OptionalLong;", &[])
		}?.l()?;

		let class = env.find_class(Self::CLASS)?;
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
	}
}

impl<'j> jni_toolbox::IntoJavaObject<'j> for crate::api::Cursor {
	const CLASS: &'static str = "mp/code/data/Cursor";
	fn into_java_object(self, env: &mut jni::JNIEnv<'j>) -> Result<jni::objects::JObject<'j>, jni::errors::Error> {
		let class = env.find_class("mp/code/data/Cursor")?;
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
	}
}

macro_rules! from_java_ptr {
	($type: ty) => {
		impl<'j> jni_toolbox::FromJava<'j> for &mut $type {
			type From = jni::sys::jobject;
			fn from_java(_env: &mut jni::JNIEnv<'j>, value: Self::From) -> Result<Self, jni::errors::Error> {
				Ok(unsafe { Box::leak(Box::from_raw(value as *mut $type)) })
			}
		}
	};
}

from_java_ptr!(crate::Client);
from_java_ptr!(crate::Workspace);
from_java_ptr!(crate::cursor::Controller);
from_java_ptr!(crate::buffer::Controller);

impl<'j> jni_toolbox::FromJava<'j> for crate::api::Config {
	type From = jni::objects::JObject<'j>;
	fn from_java(env: &mut jni::JNIEnv<'j>, config: Self::From) -> Result<Self, jni::errors::Error> {
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

impl<'j> jni_toolbox::FromJava<'j> for crate::api::Cursor {
	type From = jni::objects::JObject<'j>;
	fn from_java(env: &mut jni::JNIEnv<'j>, cursor: Self::From) -> Result<Self, jni::errors::Error> {
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

impl<'j> jni_toolbox::FromJava<'j> for crate::api::TextChange {
	type From = jni::objects::JObject<'j>;
	fn from_java(env: &mut jni::JNIEnv<'j>, change: Self::From) -> Result<Self, jni::errors::Error> {
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
