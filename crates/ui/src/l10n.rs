use guinea_plugin_l10n::L10nAccess;

pub use app_contracts::l10n::L10n;

pub fn use_tr(cx: &mut impl L10nAccess) -> L10n {
    cx.l10n::<L10n>()
}
