pub mod client;
pub mod workspace;
pub mod util;

lazy_static::lazy_static! {
	pub(crate) static ref RT: tokio::runtime::Runtime = tokio::runtime::Runtime::new().expect("could not create tokio runtime");
}

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
