fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .out_dir("src")
        .compile(&["consensus_service.proto"], &["proto/"])?;
    // tonic_build::compile_protos("proto/consensus_service.proto")?;
    Ok(())
}