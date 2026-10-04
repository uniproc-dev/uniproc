use std::path::{Path, PathBuf};
use std::process::Command;

const FILTER: &str = "crates/desktop/src/window_press/bindings.txt";
const OUTPUT: &str = "crates/desktop/src/window_press/bindings.rs";
const REACTOR: &str = "windows-reactor-pre";

pub fn run(args: &[String], workspace_root: &Path) -> anyhow::Result<()> {
    let winmd = match args.first() {
        Some(dir) => PathBuf::from(dir),
        None => anyhow::bail!(
            "pass the WinUI metadata: crates/tools/reactor/winmd of a windows-rs checkout at {}",
            reactor_commit(workspace_root).unwrap_or_else(|| format!("the commit {REACTOR} was published from"))
        ),
    };
    anyhow::ensure!(winmd.is_dir(), "no WinUI metadata at {}", winmd.display());
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

fn reactor_commit(workspace_root: &Path) -> Option<String> {
    let output = Command::new("cargo")
        .args(["metadata", "--format-version", "1"])
        .current_dir(workspace_root)
        .output()
        .ok()?;
    let metadata: serde_json::Value = serde_json::from_slice(&output.stdout).ok()?;
    let manifest = metadata["packages"]
        .as_array()?
        .iter()
        .find(|package| package["name"] == REACTOR)?["manifest_path"]
        .as_str()?
        .to_owned();
    let vcs = std::fs::read(Path::new(&manifest).parent()?.join(".cargo_vcs_info.json")).ok()?;
    let vcs: serde_json::Value = serde_json::from_slice(&vcs).ok()?;
    vcs["git"]["sha1"].as_str().map(|sha| format!("{sha} ({REACTOR})"))
}
