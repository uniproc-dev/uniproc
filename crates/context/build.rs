use build_utils::trace_scopes::{ScopeRoot, generate_trace_scopes};
use guicons_build::{Emit, IconBuild};
use std::env;
use std::path::{Path, PathBuf};

fn main() {
    generate_icons_registry();
    generate_trace_scopes_registry();
}

fn generate_icons_registry() {
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

fn generate_trace_scopes_registry() {
    // Same reasoning as icons.gui.toml above: one shared trace-scopes.toml
    // at the repo root, not owned by any one crate (see its own comment in
    // git history - it used to live under framework/, which no longer owns
    // any app-specific trace glue).
    generate_trace_scopes(
        Path::new("../../trace-scopes.toml"),
        &out_dir_file("trace_scopes.rs"),
        &[
            ScopeRoot::new("ui", "Ui"),
            ScopeRoot::new("context", "Core"),
            ScopeRoot::new("core", "Core"),
        ],
        "forsl_core::trace",
    );
}

fn out_dir() -> PathBuf {
    PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR should be set"))
}

fn out_dir_file(name: &str) -> PathBuf {
    out_dir().join(name)
}
