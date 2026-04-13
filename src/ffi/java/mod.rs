/// FFI methods relating to buffers.
pub mod buffer;

/// FFI methods relating to sessions.
pub mod session;

/// FFI methods relating to cursors.
pub mod cursor;

/// FFI methods to access extra functions.
pub mod ext;

/// FFI methods relating to the workspace.
pub mod workspace;

/// Gets or creates the relevant [tokio::runtime::Runtime].
fn tokio() -> &'static tokio::runtime::Runtime {
	use std::sync::OnceLock;
	static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
	RT.get_or_init(|| {
		tokio::runtime::Builder::new_current_thread()
			.enable_all()
			.build()
			.expect("could not create tokio runtime")
	})
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

	let level = if debug {
		tracing::Level::DEBUG
	} else {
		tracing::Level::INFO
	};

	let builder = tracing_subscriber::fmt()
		.event_format(format)
		.with_max_level(level);

	if let Some(path) = path {
		let logfile = std::fs::File::create(path).expect("failed creating logfile");
		builder.with_writer(std::sync::Mutex::new(logfile)).init();
	} else {
		builder
			.with_writer(std::sync::Mutex::new(std::io::stdout()))
			.init();
	}
}

impl jni_toolbox::IntoException for crate::errors::ConnectionError {
	fn jclass(&self) -> &'static str {
		match self {
			crate::errors::ConnectionError::Transport(_) => {
				"mp/code/exceptions/ConnectionTransportException"
			}
			crate::errors::ConnectionError::Remote(_) => {
				"mp/code/exceptions/ConnectionRemoteException"
			}
		}
	}
}

impl jni_toolbox::IntoException for crate::errors::RemoteError {
	fn jclass(&self) -> &'static str {
		"mp/code/exceptions/ConnectionRemoteException"
	}
}

impl jni_toolbox::IntoException for crate::errors::ControllerError {
	fn jclass(&self) -> &'static str {
		match self {
			crate::errors::ControllerError::Stopped => {
				"mp/code/exceptions/ControllerStoppedException"
			}
			crate::errors::ControllerError::Unfulfilled => {
				"mp/code/exceptions/ControllerUnfulfilledException"
			}
		}
	}
}

/// Generates a [jni_toolbox::IntoJava] and [jni_toolbox::FromJava] implementations
/// for a class that is just a holder for a pointer.
macro_rules! java_ptr_class {
	($type: ty, $jclass: literal) => {
		impl<'j> jni_toolbox::FromJava<'j> for &mut $type {
			type From = jni::sys::jobject;
			#[allow(unsafe_code)]
			fn from_java(
				_env: &mut jni::Env<'j>,
				value: Self::From,
			) -> Result<Self, jni::errors::Error> {
				Ok(unsafe { Box::leak(Box::from_raw(value as *mut $type)) })
			}

			fn from_jvalue(
				env: &mut jni::Env<'j>,
				value: jni::JValueOwned,
			) -> Result<Self, jni::errors::Error> {
				Self::from_java(env, value.l()?.into_raw())
			}
		}

		impl<'j> jni_toolbox::IntoJavaObject<'j> for $type {
			const CLASS: &'static str = $jclass;
			fn into_java_object(
				self,
				env: &mut jni::Env<'j>,
			) -> Result<jni::objects::JObject<'j>, jni::errors::Error> {
				let class = env.find_class(jni::strings::JNIString::new(Self::CLASS))?;
				env.new_object(
					class,
					jni::jni_sig!((ptr: i64) -> ()),
					&[jni::objects::JValue::Long(
						Box::into_raw(Box::new(self)) as jni::sys::jlong
					)],
				)
			}
		}
	};
}

java_ptr_class!(crate::prelude::CodempSession, "mp/code/Session");
java_ptr_class!(crate::prelude::CodempWorkspace, "mp/code/Workspace");
java_ptr_class!(
	crate::prelude::CodempBufferController,
	"mp/code/BufferController"
);
java_ptr_class!(
	crate::prelude::CodempCursorController,
	"mp/code/CursorController"
);
