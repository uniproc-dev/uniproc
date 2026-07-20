#![cfg_attr(coverage, feature(coverage_attribute))]
#![cfg_attr(coverage, coverage(off))]

use proc_macro::TokenStream;
use syn::{ItemImpl, parse_macro_input};

/// Thin, app-aware wrapper around `forsl_codegen::actor_manifest::
/// actor_manifest_impl`: all the generic Bus/Signals/Handlers/UI-auto-wire
/// generation lives in forsl (backend-agnostic), but reading the compiled
/// `forsl_core::contracts` registry has to happen inside a proc-macro dylib
/// that's a real Cargo dependency of `app-contracts` - forsl itself must
/// stay app-agnostic, so that lookup (and the force-link it depends on)
/// stays here instead.
#[proc_macro_attribute]
pub fn actor_manifest(attr: TokenStream, item: TokenStream) -> TokenStream {
    let impl_block = parse_macro_input!(item as ItemImpl);

    // Forces `app-contracts`'s compiled object (and its `inventory`
    // registrations) to actually be linked into this proc-macro's own
    // dylib - see the identical comment on `slint-adapter/build.rs`'s
    // `force_link_app_contracts` for the full explanation. Without this,
    // `forsl_core::contracts::bindings()` below would see an empty registry.
    let _ = std::hint::black_box(app_contracts::__force_link_anchor as fn());

    forsl_codegen::actor_manifest::actor_manifest_impl(
        attr.into(),
        impl_block,
        forsl_core::contracts::bindings(),
    )
    .into()
}
