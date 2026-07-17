use std::env;
use std::path::Path;

fn main() {
    if env::var_os("CARGO_CFG_WINDOWS").is_none() {
        return;
    }

    // Not `find_workspace_root_from`: it stops at the nearest ancestor
    // `Cargo.toml` (this crate's own), but uniproc keeps one shared
    // `icons.gui.toml` at the repo root, two levels up from `crates/desktop`.
    let manifest_path = Path::new("../../icons.gui.toml");
    let manifest = guicons_core::load_icon_manifest_or_panic(manifest_path);
    let icon_path = manifest
        .entry_for_key("sys-application-icon")
        .and_then(|entry| entry.windows_ico())
        .expect("`sys.application.icon` must define `windows-ico`");

    let mut res = winresource::WindowsResource::new();
    res.set_icon(
        icon_path
            .to_str()
            .unwrap_or_else(|| panic!("Non-UTF8 icon path: {}", icon_path.display())),
    );
    res.compile().unwrap();
}
