extern crate prost_build;


fn main() -> Result<(), Box<dyn std::error::Error>> {

    prost_build::compile_protos(&["proto/mumble.proto"], &["src/"])?;

    Ok(())
}