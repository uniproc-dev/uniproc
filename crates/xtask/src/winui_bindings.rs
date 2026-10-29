use std::path::{Path, PathBuf};
use std::process::Command;

const TARGETS: [(&str, &str); 1] = [(
    "crates/desktop/src/window_press/bindings.txt",
    "crates/desktop/src/window_press/bindings.rs",
)];
const REACTOR: &str = "windows-reactor-pre";

pub fn run(args: &[String], workspace_root: &Path) -> anyhow::Result<()> {
    let winmd = match args.first() {
        Some(dir) => PathBuf::from(dir),
        None => anyhow::bail!(
            "pass the WinUI metadata: crates/tools/reactor-metadata/winmd of a windows-rs checkout at {}",
            reactor_commit(workspace_root).unwrap_or_else(|| format!("the commit {REACTOR} was published from"))
        ),
    };
    anyhow::ensure!(winmd.is_dir(), "no WinUI metadata at {}", winmd.display());
    for (filter, output) in TARGETS {
        let mut builder = windows_bindgen::builder();
        builder
            .input(&winmd)
            .input_default()
            .output(workspace_root.join(output))
            .flat()
            .minimal()
            .filter_file(workspace_root.join(filter));
        builder.write();
        println!("{output}");
    }
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
