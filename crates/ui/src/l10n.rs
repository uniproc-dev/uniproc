use guinea::winui::Refreshable;
use windows_reactor::ViewContext;

pub use app_contracts::l10n::L10n;

pub fn tr() -> L10n {
    guinea_plugin_l10n::L10n::<L10n>::current()
}

pub fn use_tr<C: Refreshable>(cx: &mut ViewContext<C>) -> L10n {
    guinea_plugin_l10n::ui::use_l10n::<L10n, C>(cx)
}
