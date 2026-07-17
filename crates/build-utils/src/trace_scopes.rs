use std::fs;
use std::path::Path;
use toml::{Table, Value};

use crate::write_if_changed;

/// A top-level section of a `trace-scopes.toml` file (e.g. `ui`, `core`) and
/// the `ScopeKind` variant its entries should be generated with.
pub struct ScopeRoot {
    pub name: &'static str,
    pub kind: &'static str,
}

impl ScopeRoot {
    pub const fn new(name: &'static str, kind: &'static str) -> Self {
        Self { name, kind }
    }
}

/// Reads `scopes_toml`, generates `ScopeSpec` consts for every leaf boolean
/// entry under `roots`, and writes the result to `out_file`. `scope_types_path`
/// is the crate path the generated code imports `ScopeKind`/`ScopeSpec` from
/// (e.g. `"forsl_trace"`).
pub fn generate_trace_scopes(
    scopes_toml: &Path,
    out_file: &Path,
    roots: &[ScopeRoot],
    scope_types_path: &str,
) {
    println!("cargo:rerun-if-changed={}", scopes_toml.display());

    let content = fs::read_to_string(scopes_toml)
        .unwrap_or_else(|_| panic!("{} not found", scopes_toml.display()));
    let table: Table = content.parse().expect("Failed to parse trace-scopes toml");

    let mut scopes = Vec::new();
    for root in roots {
        if let Some(Value::Table(root_table)) = table.get(root.name) {
            collect_scope_entries(vec![root.name.to_string()], root.kind, root_table, &mut scopes);
        }
    }
    scopes.sort_by(|a, b| a.name.cmp(&b.name));

    let builtin_enable_scopes = policy_strings(&table, "enable_scopes");
    let builtin_disable_messages = policy_strings(&table, "disable_messages");
    let builtin_disable_targets = policy_strings(&table, "disable_targets");

    let consts = scopes
        .iter()
        .map(|entry| {
            let ctor = if entry.enabled_by_default {
                "new"
            } else {
                "disabled"
            };
            format!(
                "pub const {}: ScopeSpec = ScopeSpec::{}(\"{}\", ScopeKind::{});",
                entry.const_name, ctor, entry.name, entry.kind
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let all_scopes = scopes
        .iter()
        .map(|entry| entry.const_name.as_str())
        .collect::<Vec<_>>()
        .join(",\n    ");

    let builtin_enable_scopes = string_slice_literal(&builtin_enable_scopes);
    let builtin_disable_messages = string_slice_literal(&builtin_disable_messages);
    let builtin_disable_targets = string_slice_literal(&builtin_disable_targets);

    let generated = format!(
        r#"// AUTO-GENERATED from {toml_path}
use {scope_types_path}::{{ScopeKind, ScopeSpec}};

{consts}

pub const ALL_SCOPES: &[ScopeSpec] = &[
    {all_scopes}
];

pub const BUILTIN_ENABLE_SCOPES: &[&str] = &{builtin_enable_scopes};
pub const BUILTIN_DISABLE_MESSAGES: &[&str] = &{builtin_disable_messages};
pub const BUILTIN_DISABLE_TARGETS: &[&str] = &{builtin_disable_targets};
"#,
        toml_path = scopes_toml.display(),
    );

    write_if_changed(out_file, &generated);
}

struct ScopeEntry {
    name: String,
    const_name: String,
    kind: &'static str,
    enabled_by_default: bool,
}

fn collect_scope_entries(
    path: Vec<String>,
    root_kind: &'static str,
    table: &Table,
    acc: &mut Vec<ScopeEntry>,
) {
    for (key, value) in table {
        let mut next_path = path.clone();
        next_path.push(key.replace('-', "_"));

        match value {
            Value::Table(sub_table) => {
                collect_scope_entries(next_path, root_kind, sub_table, acc)
            }
            Value::Boolean(enabled_by_default) => {
                let name = next_path.join(".");
                let const_name = next_path
                    .iter()
                    .map(|segment| segment.to_ascii_uppercase())
                    .collect::<Vec<_>>()
                    .join("_");
                acc.push(ScopeEntry {
                    name,
                    const_name,
                    kind: root_kind,
                    enabled_by_default: *enabled_by_default,
                });
            }
            other => panic!(
                "Unexpected trace scope entry for {:?}: {other:?}",
                next_path
            ),
        }
    }
}

fn policy_strings(table: &Table, key: &str) -> Vec<String> {
    table
        .get("policy")
        .and_then(Value::as_table)
        .and_then(|policy| policy.get(key))
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn string_slice_literal(values: &[String]) -> String {
    let items = values
        .iter()
        .map(|v| format!("{v:?}"))
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{items}]")
}
