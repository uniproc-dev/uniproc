use build_utils::{slint_parser, write_if_changed};
use std::fs;
use std::path::Path;
use toml::{Table, Value};

fn main() {
    generate_slint_l10n();
    generate_capabilities_slint();
    generate_bindings_slint();

    slint_parser::generate_globals_export(Path::new("ui"));

    let mut include_paths = vec![std::path::PathBuf::from(context_icons_dir())];
    include_paths.extend([
        std::path::PathBuf::from("ui"),
        std::path::PathBuf::from("ui/shared"),
        std::path::PathBuf::from("ui/components"),
    ]);

    let config = slint_build::CompilerConfiguration::new()
        .with_style("fluent".into())
        .with_include_paths(include_paths);

    slint_build::compile_with_config("ui/app-window.slint", config).expect("Slint build failed");
}

fn context_icons_dir() -> String {
    std::env::var("DEP_CONTEXT_ICONS_ICONS_DIR")
        .expect("context's build script should publish DEP_CONTEXT_ICONS_ICONS_DIR via `links`")
}

fn generate_capabilities_slint() {
    let schema = build_utils::load_schema();
    build_utils::generate_capabilities_slint(&schema, Path::new("ui/shared/capabilities.slint"));
}

/// Bindings traits that dropped their `#[slint_bindings(global = "...")]`
/// pin get a companion global generated straight from the Rust trait -
/// no hand-authored `.slint` needed for their callbacks.
fn generate_bindings_slint() {
    let schema = build_utils::load_schema();
    for binding in schema.bindings.iter().filter(|b| b.global.is_none()) {
        let feature = feature_name_from(&binding.source_file);
        let out_file = format!("ui/features/{feature}/bindings.slint");
        build_utils::generate_binding_global_slint(binding, Path::new(&out_file));
    }
}

fn feature_name_from(source_file: &str) -> String {
    let segments: Vec<_> = Path::new(source_file)
        .iter()
        .map(|s| s.to_string_lossy().into_owned())
        .collect();
    let pos = segments
        .iter()
        .position(|s| s == "features")
        .expect("path must contain 'features'");
    segments[pos + 1].replace('-', "_")
}

fn collect_string_entries(prefix: &str, table: &Table, acc: &mut Vec<(String, String)>) {
    for (key, value) in table {
        let full_key = if prefix.is_empty() {
            key.to_string()
        } else {
            format!("{}.{}", prefix, key)
        };

        match value {
            Value::Table(sub_table) => collect_string_entries(&full_key, sub_table, acc),
            Value::String(text) => acc.push((full_key, text.clone())),
            other => acc.push((full_key, other.to_string())),
        }
    }
}

fn escape_slint_string(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn generate_slint_l10n() {
    let en_toml = Path::new("../domain/locales/en.toml");
    let out_file = Path::new("ui/shared/localization.slint");

    println!("cargo:rerun-if-changed=../domain/locales/");

    let content = fs::read_to_string(en_toml).expect("../domain/locales/en.toml not found");
    let table: Table = content.parse().expect("Failed to parse en.toml");

    let mut flat_entries = Vec::new();
    collect_string_entries("", &table, &mut flat_entries);
    flat_entries.sort_by(|a, b| a.0.cmp(&b.0));

    let properties = flat_entries
        .iter()
        .map(|(key, value)| {
            let slint_name = key.replace(['.', '_'], "-");
            let escaped = escape_slint_string(value);
            format!("    in property <string> {slint_name}: \"{escaped}\";")
        })
        .collect::<Vec<_>>()
        .join("\n");

    let generated = format!(
        "// AUTO-GENERATED — do not edit manually\nexport global L10n {{\n{properties}\n}}\n"
    );

    write_if_changed(out_file, &generated);
}
