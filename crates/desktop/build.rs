fn main() {
    windows_reactor_setup::as_self_contained();
    guinea_meta_build::generate("../../app.toml");

    let heap = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("heap.manifest");
    println!("cargo:rerun-if-changed={}", heap.display());
    println!("cargo:rustc-link-arg-bins=/MANIFESTINPUT:{}", heap.display());
}
