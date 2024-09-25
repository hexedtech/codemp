use jni_toolbox::jni;

/// Gets the current version of the Rust crate.
#[allow(non_snake_case)]
#[jni(package = "mp.code", class = "Extensions")]
fn version() -> String {
	crate::version()	
}

/// Calculate the XXH3 hash for a given String.
#[jni(package = "mp.code", class = "Extensions")]
fn hash(content: String) -> i64 {
	let hash = crate::ext::hash(content.as_bytes());
	i64::from_ne_bytes(hash.to_ne_bytes())
}

/// Tells the [tokio] runtime how to drive the event loop.
#[jni(package = "mp.code", class = "Extensions")]
fn drive(block: bool) {
	if block {
		super::tokio().block_on(std::future::pending::<()>());
	} else {
		std::thread::spawn(|| {
			super::tokio().block_on(std::future::pending::<()>());
		});
	}
}

/// Set up the tracing subscriber.
#[allow(non_snake_case)]
#[jni(package = "mp.code", class = "Extensions")]
fn setupTracing(path: Option<String>, debug: bool) {
	super::setup_logger(debug, path);
}
