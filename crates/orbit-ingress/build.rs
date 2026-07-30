fn main() -> Result<(), Box<dyn std::error::Error>> {
    // This tells Tonic to compile our proto file into Rust structs
    tonic_build::compile_protos("proto/orbit.proto")?;
    Ok(())
}