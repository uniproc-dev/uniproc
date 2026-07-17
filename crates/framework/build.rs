use build_utils::trace_scopes::{ScopeRoot, generate_trace_scopes};
use std::path::{Path, PathBuf};

fn main() {
    generate_trace_scopes(
        Path::new("./trace-scopes.toml"),
        &out_dir_file("trace_scopes.rs"),
        &[
            ScopeRoot::new("ui", "Ui"),
            ScopeRoot::new("context", "Core"),
            ScopeRoot::new("core", "Core"),
        ],
        "forsl_trace",
    );
}

fn out_dir_file(name: &str) -> PathBuf {
    PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR should be set")).join(name)
}
