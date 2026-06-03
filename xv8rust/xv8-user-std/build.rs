fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let ld_path = std::path::Path::new(&manifest_dir).join("user.ld");
    println!("cargo::rustc-link-arg=--script={}", ld_path.display());
}