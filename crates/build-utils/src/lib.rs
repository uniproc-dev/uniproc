#![cfg_attr(coverage, feature(coverage_attribute))]
#![cfg_attr(coverage, coverage(off))]
pub mod collector;
pub mod icons;

pub mod slint_parser;
pub mod trace_scopes;

pub use collector::{
    ArgDef, BindingDef, BindingMethodDef, CapabilityDef, DtoDef, DtoField, MethodDef, PortDef,
    Schema, load_schema,
};

use std::{fs, path::Path};
use strsim::jaro_winkler;

pub fn suggest_closest<'a>(
    query: &str,
    candidates: impl Iterator<Item = &'a str>,
) -> Option<&'a str> {
    candidates
        .map(|cand| (cand, jaro_winkler(query, cand)))
        .filter(|(_, sim)| *sim > 0.7)
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .map(|(cand, _)| cand)
}

pub fn write_if_changed(path: &Path, content: &str) {
    let existing = fs::read_to_string(path).unwrap_or_default();
    if existing != content {
        if let Some(p) = path.parent() {
            let _ = fs::create_dir_all(p);
        }
        fs::write(path, content).ok();
    }
}

pub fn generate_capabilities_rust(schema: &Schema, out_file: &Path) {
    let mut caps: Vec<&CapabilityDef> = schema.capabilities.iter().collect();
    caps.sort_by(|a, b| a.key.cmp(&b.key));

    let entries = caps
        .iter()
        .map(|cap| {
            let const_name = cap.key.to_uppercase().replace(['.', '-'], "_");
            format!("    pub const {}: &str = \"{}\";", const_name, cap.key)
        })
        .collect::<Vec<_>>()
        .join("\n");

    let content = format!(
        "// AUTO-GENERATED — do not edit manually\npub mod capabilities {{\n{entries}\n}}\n"
    );
    write_if_changed(out_file, &content);
}

pub fn generate_capabilities_slint(schema: &Schema, out_file: &Path) {
    let mut caps: Vec<&CapabilityDef> = schema.capabilities.iter().collect();
    caps.sort_by(|a, b| a.key.cmp(&b.key));

    let properties = caps
        .iter()
        .map(|cap| {
            let slint_name = cap.key.replace(['.', '_'], "-");
            format!("    out property <string> {}: \"{}\";", slint_name, cap.key)
        })
        .collect::<Vec<_>>()
        .join("\n");

    let content = format!(
        "// AUTO-GENERATED — do not edit manually\nexport global Capabilities {{\n{properties}\n}}\n"
    );
    write_if_changed(out_file, &content);
}
