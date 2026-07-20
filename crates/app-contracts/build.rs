fn main() {
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR missing"));

    forsl_codegen::slint_parser::generate_navigation_routes(
        std::path::Path::new("../slint-adapter/ui/pages"),
        &out_dir.join("navigation_routes.rs"),
    );

    generate_l10n_strings(&out_dir);
}

fn generate_l10n_strings(out_dir: &std::path::Path) {
    let ftl_path = std::path::Path::new("../../locales/en/main.ftl");
    println!("cargo:rerun-if-changed=../../locales");

    let messages = forsl_codegen::l10n::parse_messages(ftl_path);
    let generated = forsl_codegen::l10n::generate_strings_contract(&messages);
    let formatted = forsl_codegen::stub_gen::format_code(generated.to_string());

    std::fs::write(out_dir.join("l10n_strings.rs"), formatted)
        .expect("failed to write l10n_strings.rs");
}
