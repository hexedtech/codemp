fn main() -> Result<(), Box<dyn std::error::Error>> {
	tonic_build::compile_protos("proto/buffer.proto")?;
	tonic_build::compile_protos("proto/cursor.proto")?;
	Ok(())
}
