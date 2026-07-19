use build_utils::{slint_parser, write_if_changed};
use forsl_core::contracts::{BindingMethodMeta, BindingStubMeta};
use std::fs;
use std::path::Path;
use toml::{Table, Value};

fn main() {
    force_link_app_contracts();
    generate_slint_l10n();
    generate_capabilities_slint();
    generate_bindings_slint();
    generate_binding_adapter_bodies();

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

/// `app-contracts` is a build-dependency purely for its
/// `inventory::submit!`-registered `forsl_core::contracts` entries; nothing
/// else in this build script calls into it directly (only
/// `forsl_core::contracts::bindings()`/`ports()`, which live in `forsl-core`,
/// not `app-contracts`). The linker only pulls `.rlib` object files whose
/// symbols are actually referenced, so with zero references it would drop
/// `app-contracts`'s compiled code entirely - registrations (ctors) included
/// - and the registry would come back empty at runtime. `app_contracts::
/// __force_link_anchor` exists solely to be that one reference - dedicated
/// to this purpose, not borrowed from an unrelated port's proof function -
/// combined with the `codegen-units = 1` override for `app-contracts` in the
/// root `Cargo.toml` (so the whole crate is one object file, not 256 - one
/// reference reaches every registration, not just one feature's).
fn force_link_app_contracts() {
    let _ = std::hint::black_box(app_contracts::__force_link_anchor as fn());
}

fn context_icons_dir() -> String {
    std::env::var("DEP_CONTEXT_ICONS_ICONS_DIR")
        .expect("context's build script should publish DEP_CONTEXT_ICONS_ICONS_DIR via `links`")
}

fn generate_capabilities_slint() {
    let mut caps: Vec<_> = forsl_core::contracts::capabilities().collect();
    caps.sort_by_key(|cap| cap.key);

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
    write_if_changed(Path::new("ui/shared/capabilities.slint"), &content);
}

/// Bindings traits' callbacks get a companion `.slint` global generated
/// straight from the Rust trait (`#[bindings]`, registered via `inventory` -
/// see `forsl_core::contracts`) - no hand-authored `.slint` needed.
fn generate_bindings_slint() {
    for binding in forsl_core::contracts::bindings() {
        let out_file = format!("ui/features/{}/bindings.slint", binding.feature);
        generate_binding_global_slint(binding, Path::new(&out_file));
    }
}

fn global_name_for(binding: &BindingStubMeta) -> String {
    binding
        .trait_name
        .strip_prefix("Ui")
        .unwrap_or(binding.trait_name)
        .to_string()
}

fn adapter_type_for(binding: &BindingStubMeta) -> String {
    binding.trait_name.replace("Bindings", "Adapter")
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

fn generate_binding_global_slint(binding: &BindingStubMeta, out_file: &Path) {
    let global_name = global_name_for(binding);

    let included_methods: Vec<&BindingMethodMeta> = binding
        .methods
        .iter()
        .filter(|m| !m.slint_skip && m.slint_global_override.is_none())
        .collect();

    let mut imports: Vec<&str> = included_methods
        .iter()
        .filter_map(|m| m.slint_import)
        .collect();
    imports.sort_unstable();
    imports.dedup();
    let imports = imports.join("\n");

    let callbacks = included_methods
        .iter()
        .map(|method| {
            let slint_name = method
                .slint_name
                .map(str::to_string)
                .unwrap_or_else(|| method.name.strip_prefix("on_").unwrap_or(method.name).to_string());
            let arg_types = method
                .arg_types
                .iter()
                .enumerate()
                .map(|(i, ty)| {
                    method
                        .slint_arg_types
                        .and_then(|overrides| overrides.get(i))
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| default_slint_type(ty))
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
        binding.trait_name,
    );
    write_if_changed(out_file, &content);
}

/// Generates the *entire* `impl <Trait> for <Adapter>` block - every method,
/// manual or not. Rust doesn't allow splitting one trait's `impl` across two
/// blocks for the same type (nor does it allow `include!` to expand inside an
/// `impl`'s braces at all - only at module level), so there's no way to keep
/// a hand-written partial impl alongside a generated one. Instead, hand-written
/// files provide `#[manual]` bodies as plain inherent methods named
/// `<method>_manual` (see e.g. `features/window_actions/bindings.rs`), and
/// this generates a real top-level `impl` that either writes the full
/// `ui.global::<...>()` body (non-manual) or delegates to `self.<name>_manual(...)`
/// (manual) - both wrapped in the same upgrade-check/tracing scaffolding.
fn generate_binding_adapter_bodies() {
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR not set");
    for binding in forsl_core::contracts::bindings() {
        let out_file = Path::new(&out_dir).join(format!("{}_bindings_auto.rs", binding.feature));

        let adapter_ty = adapter_type_for(binding);
        let global_name = global_name_for(binding);

        let methods = binding
            .methods
            .iter()
            .map(|m| generate_method(&adapter_ty, &global_name, m))
            .collect::<Vec<_>>()
            .join("\n\n");

        let content = format!(
            "// AUTO-GENERATED from {trait_name} - do not edit manually\nimpl app_contracts::features::{feature}::{trait_name} for crate::features::{feature}::{adapter_ty} {{\n{methods}\n}}\n",
            trait_name = binding.trait_name,
            feature = binding.feature,
        );
        write_if_changed(&out_file, &content);
    }
}

fn generate_method(adapter_ty: &str, global_name: &str, method: &BindingMethodMeta) -> String {
    let name = method.name;
    let handler_types = method
        .arg_types
        .iter()
        .map(|ty| qualify_known_type_str(ty))
        .collect::<Vec<_>>()
        .join(", ");

    let upgrade_failure = ui_upgrade_failure_body(adapter_ty, name);
    let handler_wrap = binding_tracing_wrapper(method, name);

    let call = if method.is_manual {
        format!("        self.{name}_manual(&ui, handler);")
    } else {
        // Unlike the `.slint` global's callback declaration (which strips
        // `on_` - that's Slint's own naming convention for a callback vs.
        // its Rust subscription method), the *Rust-generated* subscription
        // method is always named `on_<callback>` - i.e. exactly the trait
        // method name, unless overridden.
        let slint_name =
            method.slint_name.map(str::to_string).unwrap_or_else(|| name.to_string());
        let global = method.slint_global_override.unwrap_or(global_name);
        let arg_idents: Vec<String> =
            (0..method.arg_types.len()).map(|i| format!("__arg{i}")).collect();
        let call_args = method
            .arg_types
            .iter()
            .enumerate()
            .map(|(i, ty)| convert_expr_from_slint(&arg_idents[i], ty))
            .collect::<Vec<_>>()
            .join(", ");
        let arg_idents_pattern = arg_idents.join(", ");
        format!(
            r#"        use slint::ComponentHandle;
        ui.global::<crate::{global}>().{slint_name}(move |{arg_idents_pattern}| {{
            handler({call_args});
        }});"#
        )
    };

    format!(
        r#"    fn {name}<F>(&self, handler: F)
    where F: Fn({handler_types}) + 'static
    {{
        let Some(ui) = self.ui.upgrade() else {{ {upgrade_failure} }};
{handler_wrap}
{call}
    }}"#
    )
}

fn binding_tracing_wrapper(method: &BindingMethodMeta, method_name: &str) -> String {
    if method.tracing_skip {
        return String::new();
    }
    let arity = method.arg_types.len();
    let scope = format!("Ui.{}", method_name);
    let target_expr = match method.tracing_target {
        Some(t) => format!("Some({t:?})"),
        None => "None".to_string(),
    };

    match arity {
        0 => format!(
            r#"        let handler = {{
            let handler = handler;
            move || forsl_core::trace::in_ui_action_scope({scope:?}, {target_expr}, None, || handler())
        }};"#
        ),
        1 => format!(
            r#"        let handler = {{
            let handler = handler;
            move |__ui_arg0| {{
                let __ui_target = forsl_core::trace::format_ui_target_1(&__ui_arg0);
                forsl_core::trace::in_ui_action_scope({scope:?}, {target_expr}, __ui_target, || handler(__ui_arg0))
            }}
        }};"#
        ),
        2 => format!(
            r#"        let handler = {{
            let handler = handler;
            move |__ui_arg0, __ui_arg1| {{
                let __ui_target = forsl_core::trace::format_ui_target_2(&__ui_arg0, &__ui_arg1);
                forsl_core::trace::in_ui_action_scope({scope:?}, {target_expr}, __ui_target, || handler(__ui_arg0, __ui_arg1))
            }}
        }};"#
        ),
        _ => panic!("binding tracing currently supports handlers with up to 2 arguments"),
    }
}

fn ui_upgrade_failure_body(adapter_ty: &str, method_name: &str) -> String {
    format!(
        r#"
            if forsl_core::trace::is_scope_enabled("ui.adapter.call") {{
                let __t = format!("{{}}::{{}}", {adapter_ty:?}, {method_name:?});
                if forsl_core::trace::is_target_enabled(&__t) {{
                    forsl_core::trace::in_named_scope("ui.adapter.call", Some("adapter,method"), Some(__t), || {{
                        tracing::error!(adapter = {adapter_ty:?}, method = {method_name:?}, "ui.adapter.upgrade_failed");
                    }});
                }}
            }}
            panic!("ui handle is dropped in {adapter_ty}::{method_name}");
        "#
    )
}

fn qualify_known_type_str(ty: &str) -> String {
    match ty {
        "SharedString" => "slint::SharedString".to_string(),
        "Image" => "slint::Image".to_string(),
        "Color" => "slint::Color".to_string(),
        _ => ty.to_string(),
    }
}

fn is_trivial_numeric(ty: &str) -> bool {
    matches!(
        ty,
        "u8" | "u16"
            | "u32"
            | "u64"
            | "u128"
            | "usize"
            | "i8"
            | "i16"
            | "i32"
            | "i64"
            | "i128"
            | "isize"
            | "f32"
            | "f64"
    )
}

fn convert_expr_from_slint(name: &str, ty: &str) -> String {
    if is_trivial_numeric(ty) {
        format!("{name} as _")
    } else if ty == "bool" {
        name.to_string()
    } else {
        format!("{name}.into()")
    }
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
