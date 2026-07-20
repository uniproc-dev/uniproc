fn main() {
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR missing"));
    println!("cargo:rerun-if-changed=../../locales");

    let ids: Vec<String> = app_contracts::features::l10n::L10N_MESSAGE_IDS
        .iter()
        .map(|id| id.to_string())
        .collect();

    let generated = forsl_codegen::l10n::generate_strings_builder(
        &ids,
        "app_contracts::features::l10n::L10nStrings",
        "crate::features::l10n::LOCALES",
    );
    let formatted = forsl_codegen::stub_gen::format_code(generated.to_string());

    std::fs::write(out_dir.join("l10n_builder.rs"), formatted)
        .expect("failed to write l10n_builder.rs");
}
