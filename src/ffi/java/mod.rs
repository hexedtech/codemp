/// FFI methods relating to buffers.
pub mod buffer;

/// FFI methods relating to clients.
pub mod client;

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


macro_rules! from_java_ptr {
	($type: ty) => {
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
	};
}

from_java_ptr!(crate::Client);
from_java_ptr!(crate::Workspace);
from_java_ptr!(crate::cursor::Controller);
from_java_ptr!(crate::buffer::Controller);

/// Generates a [JObjectify] implementation for a class that is just a holder for a pointer.
macro_rules! into_java_ptr_class {
	($type: ty, $jclass: literal) => {
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

into_java_ptr_class!(crate::Client, "mp/code/Client");
into_java_ptr_class!(crate::Workspace, "mp/code/Workspace");
into_java_ptr_class!(crate::cursor::Controller, "mp/code/CursorController");
into_java_ptr_class!(crate::buffer::Controller, "mp/code/BufferController");

// #[allow(unsafe_code)]
impl<'j> jni_toolbox::IntoJavaObject<'j> for crate::proto::workspace::WorkspaceEventKind { // TODO
	const CLASS: &'static str = "mp/code/Workspace$Event";
	fn into_java_object(
		self,
		env: &mut jni::Env<'j>,
	) -> Result<jni::objects::JObject<'j>, jni::errors::Error> {
		let (ordinal, user, buffer) = match self {
			crate::api::Event::UserJoin { name } => (0, Some(name), None),
			crate::api::Event::UserLeave { name } => (1, Some(name), None),
			crate::api::Event::FileTreeUpdated { path } => (2, None, Some(path)),
			crate::api::Event::UserJoinBuffer { name, buffer } => (3, Some(name), Some(buffer)),
			crate::api::Event::UserLeaveBuffer { name, buffer } => (4, Some(name), Some(buffer)),
		};

		let type_class = env.find_class(jni::jni_str!("mp/code/Workspace$Event$Type"))?;
		let variants = env
			.call_method(type_class, jni::jni_str!("getEnumConstants"), jni::jni_sig!("()[Ljava/lang/Object;"), &[])?
			.l()?;
		let variants_array = jni::objects::JObjectArray::<jni::objects::JObject>::cast_local(env, variants)?;
		let event_type = variants_array.get_element(env, ordinal)?;

		let class_name = jni::strings::JNIString::new(Self::CLASS);
		let event_class = env.find_class(class_name)?;

		let j_event_type = event_type.into_java_object(env)?;
		let j_user = user.into_java_object(env)?;
		let j_buffer = buffer.into_java_object(env)?;

		env.new_object(
			event_class,
			jni::jni_sig!("(Lmp/code/Workspace$Event$Type;Ljava/lang/String;)V"),
			&[
				jni::JValue::Object(&j_event_type),
				jni::JValue::Object(&j_user),
				jni::JValue::Object(&j_buffer)
			]
		)
	}
}
