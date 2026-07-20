use fluent_templates::Loader;
use fluent_templates::static_loader;

static_loader! {
    pub static LOCALES = {
        locales: "../../locales",
        fallback_language: "en",
    };
}

include!(concat!(env!("OUT_DIR"), "/l10n_builder.rs"));
