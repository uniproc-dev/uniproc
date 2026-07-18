#![cfg_attr(coverage, feature(coverage_attribute))]
#![cfg_attr(coverage, coverage(off))]
pub mod collector;

pub mod slint_parser;

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

/// The Slint global name a binding's callbacks get generated into when its
/// trait doesn't pin one via `#[slint_bindings(global = "...")]`: strip the
/// `Ui` prefix from the trait name (e.g. `UiSidebarBindings` -> `SidebarBindings`).
pub fn binding_default_global_name(binding_trait_name: &str) -> String {
    binding_trait_name
        .strip_prefix("Ui")
        .unwrap_or(binding_trait_name)
        .to_string()
}

/// Generates a `.slint` global exposing one `callback` per method of a
/// binding trait that opted out of a hand-written global (`global: None` in
/// the schema) - see [`binding_default_global_name`] for the global's name.
/// Only meant for traits with no state properties of their own; anything
/// with `in`/`in-out` properties still needs a hand-written global.
pub fn generate_binding_global_slint(binding: &BindingDef, out_file: &Path) {
    let global_name = binding_default_global_name(&binding.name);

    let included_methods: Vec<_> = binding
        .methods
        .iter()
        .filter(|m| !m.slint_skip && m.global_override.is_none())
        .collect();

    let mut imports: Vec<&str> = included_methods
        .iter()
        .filter_map(|m| m.slint_import.as_deref())
        .collect();
    imports.sort_unstable();
    imports.dedup();
    let imports = imports.join("\n");

    let callbacks = included_methods
        .iter()
        .map(|method| {
            // `method.name` follows Slint's own `on_<callback>` subscription-method
            // convention (e.g. `on_side_bar_width_changed`); the callback itself is
            // declared under the bare name. `slint_name` overrides the bare name
            // directly for cases where that convention doesn't hold.
            let slint_name = method
                .slint_name
                .clone()
                .unwrap_or_else(|| method.name.strip_prefix("on_").unwrap_or(&method.name).to_string());
            let arg_types = method
                .handler_args
                .iter()
                .enumerate()
                .map(|(i, arg)| {
                    method
                        .slint_arg_types
                        .as_ref()
                        .and_then(|overrides| overrides.get(i))
                        .cloned()
                        .unwrap_or_else(|| default_slint_type(&arg.ty))
                })
                .collect::<Vec<_>>()
                .join(", ");
            format!("    callback {slint_name}({arg_types});")
        })
        .collect::<Vec<_>>()
        .join("\n");

    let imports_block = if imports.is_empty() {
        String::new()
    } else {
        format!("{imports}\n\n")
    };

    let content = format!(
        "// AUTO-GENERATED from {} - do not edit manually\n{imports_block}export global {global_name} {{\n{callbacks}\n}}\n",
        binding.source_file,
    );
    write_if_changed(out_file, &content);
}

fn default_slint_type(rust_ty: &str) -> String {
    match rust_ty {
        "u8" | "u16" | "u32" | "u64" | "u128" | "usize" | "i8" | "i16" | "i32" | "i64" | "i128"
        | "isize" => "int".to_string(),
        "f32" | "f64" => "float".to_string(),
        "bool" => "bool".to_string(),
        "String" | "SharedString" => "string".to_string(),
        other => other.to_string(),
    }
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
