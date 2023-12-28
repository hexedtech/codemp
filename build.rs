fn main() -> Result<(), Box<dyn std::error::Error>> {
	tonic_build::configure()
		.build_server(false)
		.compile(
			&[
				"proto/model/cursor.proto",
				"proto/model/user.proto",
				"proto/buffer_service.proto",
				"proto/cursor_service.proto",
				"proto/workspace_service.proto"
			],
			&["proto", "proto", "proto","proto", "proto"]
				)?;
	Ok(())
 }