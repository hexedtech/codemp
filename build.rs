fn main() -> Result<(), Box<dyn std::error::Error>> {
	tonic_build::configure()
		// .build_client(cfg!(feature = "client"))
		//.build_server(cfg!(feature = "server")) // FIXME if false, build fails????
		// .build_transport(cfg!(feature = "transport"))
		.compile(
			&[
				"proto/user.proto",
				"proto/cursor.proto",
				"proto/buffer_service.proto",
				"proto/cursor_service.proto",
				"proto/workspace_service.proto"
			],
			&["proto"]
		)?;
	Ok(())
 }