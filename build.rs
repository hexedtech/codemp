fn main() -> Result<(), Box<dyn std::error::Error>> {
	tonic_build::configure()
		// .build_client(cfg!(feature = "client"))
		// .build_server(cfg!(feature = "server")) // FIXME if false, build fails????
		// .build_transport(cfg!(feature = "transport"))
		.compile(
			&[
				"proto/common.proto",
				"proto/cursor.proto",
				"proto/files.proto",
				"proto/auth.proto",
				"proto/workspace.proto",
				"proto/buffer.proto",
			],
			&["proto"],
		)?;
	Ok(())
 }
