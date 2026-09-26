use guicons_build::{Emit, IconBuild};

fn main() {
    guinea_plugin_l10n_build::build("../../locales");

    IconBuild::auto().emit(Emit::Rust).build();
}
