use std::path::PathBuf;

fn main() {
    generate_icons_registry();
}

fn generate_icons_registry() {
    let out = out_dir_file("icons.rs");
    build_utils::icons::IconBuild::auto()
        .emit_shared_bundle()
        .emit_rust_registry(out)
        .run();
}

fn out_dir_file(name: &str) -> PathBuf {
    PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR should be set")).join(name)
}
