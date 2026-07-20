use forsl_codegen::bindings_gen;
use forsl_codegen::{slint_parser, write_if_changed};
use std::path::Path;

const CONTRACTS_CRATE: &str = "app_contracts";
const ADAPTER_PATH: &str = "crate::features";

fn main() {
    force_link_app_contracts();
    generate_slint_l10n();
    generate_l10n_load_body();
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
    let content = bindings_gen::generate_capabilities_slint_content(forsl_core::contracts::capabilities());
    write_if_changed(Path::new("ui/shared/capabilities.slint"), &content);
}

/// Bindings traits' callbacks get a companion `.slint` global generated
/// straight from the Rust trait (`#[bindings]`, registered via `inventory` -
/// see `forsl_core::contracts`) - no hand-authored `.slint` needed.
fn generate_bindings_slint() {
    for binding in forsl_core::contracts::bindings() {
        let out_file = format!("ui/features/{}/bindings.slint", binding.feature);
        let content = bindings_gen::generate_binding_global_slint_content(binding);
        write_if_changed(Path::new(&out_file), &content);
    }
}

/// Generates the *entire* `impl <Trait> for <Adapter>` block per feature -
/// see `forsl_codegen::bindings_gen::generate_binding_adapter_impl_content`
/// for why hand-written logic can't just be merged into one impl block by
/// hand, and instead lives as `<method>_manual` inherent methods that the
/// generated body delegates to.
fn generate_binding_adapter_bodies() {
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR not set");
    for binding in forsl_core::contracts::bindings() {
        let out_file = Path::new(&out_dir).join(format!("{}_bindings_auto.rs", binding.feature));
        let content =
            bindings_gen::generate_binding_adapter_impl_content(binding, CONTRACTS_CRATE, ADAPTER_PATH);
        write_if_changed(&out_file, &content);
    }
}

fn l10n_message_ids() -> Vec<String> {
    app_contracts::features::l10n::L10N_MESSAGE_IDS
        .iter()
        .map(|id| id.to_string())
        .collect()
}

fn l10n_message_defaults() -> Vec<(String, String)> {
    app_contracts::features::l10n::L10N_MESSAGE_DEFAULTS
        .iter()
        .map(|(id, text)| (id.to_string(), text.to_string()))
        .collect()
}

fn generate_slint_l10n() {
    let content = forsl_codegen::l10n::generate_slint_global_content(&l10n_message_defaults());
    write_if_changed(Path::new("ui/shared/localization.slint"), &content);
}

fn generate_l10n_load_body() {
    let content = forsl_codegen::l10n::generate_load_body_content(&l10n_message_ids());
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR not set");
    write_if_changed(&Path::new(&out_dir).join("l10n_load_body.rs"), &content);
}
