use guicons_build::{Emit, IconBuild};
use std::env;
use std::path::{Path, PathBuf};

fn main() {
    // Not `IconBuild::auto()`: that stops at the nearest ancestor `Cargo.toml`
    // (this crate's own), but uniproc keeps one shared `icons.gui.toml` at the
    // repo root, two levels up from `crates/context`.
    IconBuild::new(Path::new("../../icons.gui.toml"))
        .emit(Emit::Rust)
        .emit(Emit::Slint)
        .build();

    // Published to dependents (slint-adapter) via the `links` mechanism so
    // their build scripts can add this crate's OUT_DIR as a Slint include
    // path and find the generated `icons.slint`.
    println!("cargo:icons_dir={}", out_dir().display());
}

fn out_dir() -> PathBuf {
    PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR should be set"))
}
