use guinea::feature::FeatureInitContext;
use guinea::winui::{page, Page, PageCx, UpdateCx};
use ui::pages::processes::{ProcessesSettingsMsg, ProcessesSettingsPage};
use ui::theme::{scheme_context, Palette};
use windows_reactor::{Callback, View};

use super::processes::open_settings;
use crate::routes::Route;

#[derive(Default)]
pub struct ProcessesSettings(ProcessesSettingsPage);

#[page]
impl Page for ProcessesSettings {
    type Params = crate::routes::ProcessesSettingsParams;
    type Installs = ();
    type Message = ProcessesSettingsMsg;

    fn install(_ctx: &FeatureInitContext, _params: &Self::Params) -> anyhow::Result<Self::Installs> {
        Ok(())
    }

    fn init(_ctx: &FeatureInitContext, _params: &Self::Params) -> Self {
        Self(ProcessesSettingsPage::new(open_settings()))
    }

    fn update(&mut self, message: ProcessesSettingsMsg, _cx: &mut UpdateCx<'_, Self>) {
        self.0.update(message);
    }

    fn view(&self, cx: &mut PageCx<'_, Self>) -> View {
        let l10n = ui::l10n::use_tr(cx);
        let palette = Palette::of(cx.use_context(scheme_context()));
        let forward = cx.on(|message: ProcessesSettingsMsg| message);
        let nav = cx.navigate::<Route>();
        let back = Callback::new(move |()| nav.to(Route::Processes {}));
        self.0.view(&l10n, palette, forward, back)
    }
}

#[cfg(test)]
mod tests {
    use app_contracts::features::processes::{ProcessCategory, ProcessColumn};
    use domain::features::processes::settings::ProcessesSettings as Stored;
    use guinea::app::Harness;
    use guinea::winui::harness::{Mounted, Node, PropertyId, PropertyValue};
    use guinea::Mark;
    use guinea_plugin_l10n::L10nPlugin;
    use guinea_plugin_store::amethystate::store::builder::Backend;
    use guinea_plugin_store::StorePlugin;
    use ui::pages::processes::{ProcessesSettingsMark, SectionId};

    use super::*;

    fn start(h: &mut Harness) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        h.plugin(StorePlugin::at(dir.path().join("settings")).backend(Backend::Json))
            .unwrap()
            .plugin(L10nPlugin::<app_contracts::l10n::L10n>::new("en"))
            .unwrap();
        dir
    }

    fn mount(h: &Harness) -> Mounted<'_, ProcessesSettings> {
        let params = crate::routes::ProcessesSettingsParams::default();
        let mut page = Mounted::mount_at(&h.child(), params, Route::ProcessesSettings {}).unwrap();
        page.settle();
        page
    }

    fn marked(node: &Node, wanted: &[&str], out: &mut Vec<String>) {
        if let Some(id) = node.id.as_deref().filter(|id| wanted.contains(id)) {
            out.push(id.to_string());
        }
        for child in &node.children {
            marked(child, wanted, out);
        }
    }

    fn columns(page: &Mounted<'_, ProcessesSettings>) -> Vec<String> {
        let names = ProcessColumn::ALL.map(|column| column.name());
        let mut out = Vec::new();
        marked(&page.tree(), &names, &mut out);
        out
    }

    fn all_sections() -> Vec<SectionId> {
        std::iter::once(SectionId::Pinned)
            .chain(ProcessCategory::ORDER.map(SectionId::Category))
            .collect()
    }

    fn sections(page: &Mounted<'_, ProcessesSettings>) -> Vec<String> {
        let names: Vec<&str> = all_sections().iter().map(|section| section.name()).collect();
        let mut out = Vec::new();
        marked(&page.tree(), &names, &mut out);
        out
    }

    fn names(marks: &[impl Mark]) -> Vec<String> {
        marks.iter().map(|mark| mark.name().to_string()).collect()
    }

    fn property(
        page: &mut Mounted<'_, ProcessesSettings>,
        row: impl Mark,
        mark: ProcessesSettingsMark,
        property: PropertyId,
    ) -> Option<PropertyValue> {
        let node = page.within(row).find(mark)?;
        page.property(node, property).cloned()
    }

    fn enabled(page: &mut Mounted<'_, ProcessesSettings>, row: impl Mark, mark: ProcessesSettingsMark) -> bool {
        property(page, row, mark, PropertyId::ButtonIsEnabled) != Some(PropertyValue::Bool(false))
    }

    fn shown(page: &mut Mounted<'_, ProcessesSettings>, column: ProcessColumn) -> Option<PropertyValue> {
        property(page, column, ProcessesSettingsMark::Shown, PropertyId::ToggleSwitchIsOn)
    }

    #[guinea::test(iterations = 4, exclusive = "store")]
    fn every_column_is_listed_in_table_order_with_whether_it_is_shown(h: &mut Harness) {
        use ProcessColumn::*;
        let _store = start(h);
        let h = &*h;
        let mut page = mount(h);

        assert_eq!(columns(&page), names(&ProcessColumn::ALL));
        assert_eq!(shown(&mut page, Pid), Some(PropertyValue::Bool(false)));
        assert_eq!(shown(&mut page, Cpu), Some(PropertyValue::Bool(true)));
        assert_eq!(
            property(&mut page, Name, ProcessesSettingsMark::Shown, PropertyId::ToggleSwitchIsEnabled),
            Some(PropertyValue::Bool(false)),
            "the name column cannot be hidden"
        );
    }

    #[guinea::test(iterations = 4, exclusive = "store")]
    fn the_name_column_stays_first(h: &mut Harness) {
        use ProcessColumn::*;
        let _store = start(h);
        let h = &*h;
        let mut page = mount(h);

        assert!(!enabled(&mut page, Name, ProcessesSettingsMark::Up));
        assert!(!enabled(&mut page, Name, ProcessesSettingsMark::Down));
        assert!(!enabled(&mut page, Pid, ProcessesSettingsMark::Up));
        assert!(enabled(&mut page, Pid, ProcessesSettingsMark::Down));
        assert!(!enabled(&mut page, Disk, ProcessesSettingsMark::Down));
    }

    #[guinea::test(iterations = 4, exclusive = "store")]
    fn a_column_moved_down_trades_places_and_stays_there(h: &mut Harness) {
        use ProcessColumn::*;
        let _store = start(h);
        let h = &*h;
        let mut page = mount(h);

        page.within(Cpu).click(ProcessesSettingsMark::Down).settle();
        page.settle();

        assert_eq!(columns(&page), names(&[Name, Pid, ProcessName, Memory, Cpu, Net, Disk]));
        let ranks = Stored::new().unwrap().columns().order();
        assert_eq!(ranks.get("memory"), Some(3));
        assert_eq!(ranks.get("cpu"), Some(4));
    }

    #[guinea::test(iterations = 4, exclusive = "store")]
    fn a_column_shown_here_is_kept_shown(h: &mut Harness) {
        use ProcessColumn::*;
        let _store = start(h);
        let h = &*h;
        let mut page = mount(h);

        page.within(Pid).click(ProcessesSettingsMark::Shown).settle();
        page.settle();

        assert_eq!(shown(&mut page, Pid), Some(PropertyValue::Bool(true)));
        let stored = Stored::new().unwrap().columns().configs().get(Pid.id());
        assert!(stored.is_some_and(|config| config.visible));

        page.within(Pid).click(ProcessesSettingsMark::Shown).settle();
        page.settle();

        assert_eq!(shown(&mut page, Pid), Some(PropertyValue::Bool(false)));
        let stored = Stored::new().unwrap().columns().configs().get(Pid.id());
        assert!(stored.is_some_and(|config| !config.visible));
    }

    #[guinea::test(iterations = 4, exclusive = "store")]
    fn a_section_moves_up_and_the_order_can_be_reset(h: &mut Harness) {
        let _store = start(h);
        let h = &*h;
        let mut page = mount(h);
        let services = SectionId::Category(ProcessCategory::WindowsService);
        let background = SectionId::Category(ProcessCategory::BackgroundMicrosoft);
        assert_eq!(sections(&page), names(&all_sections()));
        assert!(!enabled(&mut page, ProcessesSettingsMark::ResetSections, ProcessesSettingsMark::ResetSections));

        page.within(services).click(ProcessesSettingsMark::Up).settle();
        page.settle();

        let moved = sections(&page);
        let at = |section: SectionId| moved.iter().position(|name| name == section.name());
        assert_eq!(at(services).map(|at| at + 1), at(background), "{moved:?}");
        let ranks = Stored::new().unwrap().grouping().section_order();
        assert_eq!(ranks.get(services.name()), at(services).map(|at| at as u32));

        page.click(ProcessesSettingsMark::ResetSections).settle();
        page.settle();

        assert_eq!(sections(&page), names(&all_sections()));
        let ranks = Stored::new().unwrap().grouping().section_order();
        assert_eq!(ranks.get(services.name()), None);
    }

    #[guinea::test(iterations = 4, exclusive = "store")]
    fn the_breadcrumb_leads_back_to_the_processes(h: &mut Harness) {
        let _store = start(h);
        let h = &*h;
        let mut page = mount(h);

        page.click(ProcessesSettingsMark::Back).settle();
        page.settle();

        assert_eq!(page.navigated::<Route>(), [Route::Processes {}]);
    }
}
