use std::path::{Path, PathBuf};
use std::process::Command;

const FILTER: &str = "crates/desktop/src/window_press/bindings.txt";
const OUTPUT: &str = "crates/desktop/src/window_press/bindings.rs";

pub fn run(workspace_root: &Path) -> anyhow::Result<()> {
    let winmd = reactor_winmd(workspace_root)?;
    windows_bindgen::builder()
        .input(&winmd)
        .input_default()
        .output(workspace_root.join(OUTPUT))
        .minimal()
        .flat()
        .filter_file(workspace_root.join(FILTER))
        .write();
    Ok(())
}

fn reactor_winmd(workspace_root: &Path) -> anyhow::Result<PathBuf> {
    let output = Command::new("cargo")
        .args(["metadata", "--format-version", "1"])
        .current_dir(workspace_root)
        .output()?;
    anyhow::ensure!(output.status.success(), "cargo metadata failed");

    let metadata: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    let manifest = metadata["packages"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|package| package["name"] == "windows-reactor")
        .and_then(|package| package["manifest_path"].as_str())
        .ok_or_else(|| anyhow::anyhow!("windows-reactor is not in the dependency graph"))?;

    let winmd = Path::new(manifest)
        .ancestors()
        .nth(4)
        .map(|repo| repo.join("crates/tools/reactor/winmd"))
        .ok_or_else(|| anyhow::anyhow!("cannot resolve the windows-rs checkout from {manifest}"))?;
    anyhow::ensure!(winmd.is_dir(), "no WinUI metadata at {}", winmd.display());
    Ok(winmd)
}
