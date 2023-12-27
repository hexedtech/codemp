fn main() -> Result<(), Box<dyn std::error::Error>> {
	tonic_build::compile_protos("proto/model/cursor.proto")?;
	tonic_build::compile_protos("proto/model/user.proto")?;
	tonic_build::compile_protos("proto/buffer_service.proto")?;
	tonic_build::compile_protos("proto/cursor_service.proto")?;
	tonic_build::compile_protos("proto/workspace_service.proto")?;
	Ok(())
}
