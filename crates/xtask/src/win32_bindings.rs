use std::path::{Path, PathBuf};

#[derive(Clone, Copy)]
enum Shape {
    Whole,
    Minimal,
}

const BINDINGS: [(&str, &str, Shape); 4] = [
    ("crates/context/src/bindings.txt", "crates/context/src/bindings.rs", Shape::Whole),
    ("crates/domain/src/bindings.txt", "crates/domain/src/bindings.rs", Shape::Whole),
    ("crates/ui/src/bindings.txt", "crates/ui/src/bindings.rs", Shape::Minimal),
    ("crates/xtask/src/bindings.txt", "crates/xtask/src/bindings.rs", Shape::Whole),
];

pub fn run(workspace_root: &Path) -> anyhow::Result<()> {
    for (filter, output, shape) in pairs(workspace_root) {
        generate(&filter, &output, shape);
        println!("{}", output.display());
    }
    Ok(())
}

fn pairs(workspace_root: &Path) -> Vec<(PathBuf, PathBuf, Shape)> {
    BINDINGS
        .iter()
        .map(|&(filter, output, shape)| (workspace_root.join(filter), workspace_root.join(output), shape))
        .collect()
}

fn generate(filter: &Path, output: &Path, shape: Shape) {
    let mut builder = windows_bindgen::builder();
    builder.input_default().flat().filter_file(filter).output(output);
    if let Shape::Minimal = shape {
        builder.minimal();
    }
    builder.write();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn squeezed(text: &str) -> String {
        text.split_whitespace().collect()
    }

    #[test]
    fn every_crate_s_bindings_are_what_its_filter_writes() {
        let root = crate::workspace_root().expect("the workspace root");
        for (index, (filter, output, shape)) in pairs(&root).into_iter().enumerate() {
            let fresh = std::env::temp_dir().join(format!("uniproc-bindings-{}-{index}.rs", std::process::id()));
            generate(&filter, &fresh, shape);
            let written = std::fs::read_to_string(&fresh).unwrap_or_default();
            let _ = std::fs::remove_file(&fresh);
            let held = std::fs::read_to_string(&output).unwrap_or_default();
            assert!(
                !written.is_empty() && squeezed(&held) == squeezed(&written),
                "{} is not what {} writes: run `cargo run -p xtask -- bindings`",
                output.display(),
                filter.display()
            );
        }
    }
}
