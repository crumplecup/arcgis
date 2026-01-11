//! Build script for generating Rust code from protocol buffer schemas.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generate Rust code from the ArcGIS FeatureCollection proto schema
    prost_build::Config::new()
        .out_dir("src/services/feature/pbf")
        .compile_protos(&["proto/FeatureCollection.proto"], &["proto/"])?;

    // Tell Cargo to rerun this build script if the proto file changes
    println!("cargo:rerun-if-changed=proto/FeatureCollection.proto");

    Ok(())
}
