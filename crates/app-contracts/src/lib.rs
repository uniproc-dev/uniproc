pub mod features;

/// Exists purely so build scripts that need this crate's compiled object
/// linked in (to keep its `inventory::submit!` registrations from being
/// dropped as unreferenced - see `forsl_core::contracts` and
/// `slint-adapter/build.rs`'s `force_link_app_contracts`) have a stable,
/// dedicated symbol to reference, instead of piggybacking on an unrelated
/// port's proof function.
#[doc(hidden)]
pub fn __force_link_anchor() {}
