use app_contracts::features::settings::SettingsState;
use guinea::feature::FeatureInitContext;
use guinea::winui::{page, Page, PageCx, UpdateCx};
use ui::theme::{scheme_context, Palette};
use windows_reactor::View;

#[derive(Default)]
pub struct Settings;

#[page]
impl Page for Settings {
    type Params = crate::routes::SettingsParams;
    type Installs = ();
    type Message = ();

    fn install(_ctx: &FeatureInitContext, _params: &Self::Params) -> anyhow::Result<Self::Installs> {
        Ok(())
    }

    fn update(&mut self, _message: (), _cx: &mut UpdateCx<'_, Self>) {}

    fn view(&self, cx: &mut PageCx<'_, Self>) -> View {
        let (state, dispatch) = cx.use_reducer::<SettingsState, _>();
        let l10n = ui::l10n::use_tr(cx);
        let palette = Palette::of(cx.use_context(scheme_context()));
        ui::pages::settings::settings_view(&state, &dispatch, &l10n, palette)
    }
}

#[cfg(test)]
mod tests {
    use app_contracts::features::settings::{
        AppTheme, SetStartPage, SetTheme, SetUpdateInterval, StartPage, UpdateIntervalChanged,
    };
    use domain::features::settings::settings::GeneralSettings;
    use domain::features::settings::SettingsFeature;
    use guinea::app::Harness;
    use guinea::winui::harness::{Mounted, PropertyId, PropertyValue};
    use guinea_plugin_l10n::L10nPlugin;
    use guinea_plugin_store::amethystate::store::builder::Backend;
    use guinea_plugin_store::StorePlugin;
    use ui::pages::settings::SettingsMark;

    use super::*;

    fn start(h: &mut Harness) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        h.plugin(StorePlugin::at(dir.path().join("settings")).backend(Backend::Json))
            .unwrap()
            .plugin(L10nPlugin::<app_contracts::l10n::L10n>::new("en"))
            .unwrap();
        dir
    }

    fn mount(h: &Harness) -> Mounted<'_, Settings> {
        h.install::<SettingsFeature>(&()).unwrap();
        let mut page = Mounted::<Settings>::mount(&h.child(), crate::routes::SettingsParams::default()).unwrap();
        page.settle();
        page
    }

    fn value(page: &Mounted<'_, Settings>, mark: SettingsMark, property: PropertyId) -> PropertyValue {
        let node = page.find(mark).unwrap_or_else(|| panic!("{mark:?}: {:#?}", page.tree()));
        page.property(node, property).cloned().unwrap_or_else(|| panic!("{mark:?} has no {property:?}"))
    }

    fn shown_interval(page: &Mounted<'_, Settings>) -> Option<String> {
        page.tree().find(SettingsMark::UpdateSpeedValue).and_then(|node| node.text.clone())
    }

    #[guinea::test(iterations = 4, exclusive = "store")]
    fn the_page_shows_the_defaults(h: &mut Harness) {
        let _store = start(h);
        let h = &*h;
        let page = mount(h);

        assert_eq!(value(&page, SettingsMark::Theme, PropertyId::ComboBoxSelectedIndex), PropertyValue::SelectionIndex(Some(0)));
        assert_eq!(value(&page, SettingsMark::StartPage, PropertyId::ComboBoxSelectedIndex), PropertyValue::SelectionIndex(Some(0)));
        assert_eq!(value(&page, SettingsMark::UpdateSpeed, PropertyId::SliderValue), PropertyValue::F64(1500.0));
        assert_eq!(value(&page, SettingsMark::UpdateSpeed, PropertyId::SliderMinimum), PropertyValue::F64(100.0));
        assert_eq!(value(&page, SettingsMark::UpdateSpeed, PropertyId::SliderMaximum), PropertyValue::F64(5000.0));
        assert_eq!(shown_interval(&page).as_deref(), Some("\u{2068}1.5\u{2069} s"));
        assert!(page.find_text("Use system setting").is_none(), "a closed choice shows no list");
    }

    #[guinea::test(iterations = 4, exclusive = "store")]
    fn choices_are_shown_and_kept(h: &mut Harness) {
        let _store = start(h);
        let h = &*h;
        let mut page = mount(h);

        let dispatch = h.dispatch::<SettingsState>();
        dispatch.emit(SetTheme(AppTheme::Dark));
        dispatch.emit(SetStartPage(StartPage::Wsl));
        dispatch.emit(SetUpdateInterval(300));
        page.settle();

        assert_eq!(value(&page, SettingsMark::Theme, PropertyId::ComboBoxSelectedIndex), PropertyValue::SelectionIndex(Some(2)));
        assert_eq!(value(&page, SettingsMark::StartPage, PropertyId::ComboBoxSelectedIndex), PropertyValue::SelectionIndex(Some(3)));
        assert_eq!(value(&page, SettingsMark::UpdateSpeed, PropertyId::SliderValue), PropertyValue::F64(300.0));
        assert_eq!(shown_interval(&page).as_deref(), Some("\u{2068}300\u{2069} ms"));

        let stored = GeneralSettings::new().unwrap();
        assert_eq!(stored.theme_choice(), AppTheme::Dark);
        assert_eq!(stored.start_page_choice(), StartPage::Wsl);
        assert_eq!(stored.update_interval_ms().get(), 300);
    }

    #[guinea::test(iterations = 4, exclusive = "store")]
    fn a_new_interval_is_announced_once_and_inside_the_range(h: &mut Harness) {
        let _store = start(h);
        let h = &*h;
        let mut page = mount(h);

        let first = h.act::<SettingsState>(SetUpdateInterval(9_000));
        first.settle();
        assert!(first.chain().published::<UpdateIntervalChanged>(), "{:#?}", first.chain());
        page.settle();
        assert_eq!(h.state::<SettingsState>().update_interval_ms, 5_000);
        assert_eq!(shown_interval(&page).as_deref(), Some("\u{2068}5\u{2069} s"));

        let again = h.act::<SettingsState>(SetUpdateInterval(5_000));
        again.settle();
        assert!(!again.chain().published::<UpdateIntervalChanged>(), "the same interval is not news");
    }
}
