// See: https://crates.io/crates/built
fn main() {
    // Inject git tag as the version number to override the one in Cargo.toml
    // See: https://github.com/rust-lang/cargo/issues/6583#issuecomment-1259871885
    if let Ok(val) = std::env::var("ODS_RELEASE_VERSION") {
        println!("cargo:rustc-env=CARGO_PKG_VERSION={}", val);
    }
    println!("cargo:rerun-if-env-changed=ODS_RELEASE_VERSION");
    built::write_built_file().expect("Failed to acquire build-time information")
}
