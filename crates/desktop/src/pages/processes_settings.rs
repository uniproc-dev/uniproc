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

    fn init(ctx: &FeatureInitContext, _params: &Self::Params) -> Self {
        Self(ProcessesSettingsPage::new(open_settings(ctx)))
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
    use guinea_plugin_store::{StoreAccess, StorePlugin};
    use ui::pages::processes::{Group, ProcessesSettingsMark, SectionId};

    use super::*;

    fn stored(h: &Harness) -> Stored {
        h.segment().settings::<Stored>().unwrap()
    }

    fn start(h: &mut Harness) {
        h.plugin(StorePlugin::in_memory())
            .unwrap()
            .plugin(L10nPlugin::<app_contracts::l10n::L10n>::new("en"))
            .unwrap();
    }

    fn mount(h: &Harness) -> Mounted<'_, ProcessesSettings> {
        let params = crate::routes::ProcessesSettingsParams::default();
        let mut page = Mounted::mount_at(h.child(), params, Route::ProcessesSettings {}).unwrap();
        page.settle();
        page
    }

    fn expanded(page: &mut Mounted<'_, ProcessesSettings>, group: ProcessesSettingsMark) -> Option<PropertyValue> {
        let node = page.find(group)?;
        page.property(node, PropertyId::ExpanderIsExpanded).cloned()
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

    #[guinea::test(iterations = 4)]
    fn the_groups_start_closed_and_stay_as_they_were_left(h: &mut Harness) {
        use ProcessesSettingsMark::{ColumnsGroup, SectionsGroup};
        start(h);
        let h = &*h;
        let mut page = mount(h);
        assert_eq!(expanded(&mut page, ColumnsGroup), Some(PropertyValue::Bool(false)));
        assert_eq!(expanded(&mut page, SectionsGroup), Some(PropertyValue::Bool(false)));

        page.send(ProcessesSettingsMsg::Expand(Group::Columns, true));
        page.settle();
        page.within(ProcessColumn::Pid).click(ProcessesSettingsMark::Shown).settle();
        page.settle();
        assert_eq!(
            expanded(&mut page, ColumnsGroup),
            Some(PropertyValue::Bool(true)),
            "a redraw keeps the group open"
        );
        assert_eq!(expanded(&mut page, SectionsGroup), Some(PropertyValue::Bool(false)));

        page.send(ProcessesSettingsMsg::Expand(Group::Columns, false));
        page.settle();
        assert_eq!(expanded(&mut page, ColumnsGroup), Some(PropertyValue::Bool(false)));
    }

    #[guinea::test(iterations = 4)]
    fn every_column_is_listed_in_table_order_with_whether_it_is_shown(h: &mut Harness) {
        use ProcessColumn::*;
        start(h);
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

    #[guinea::test(iterations = 4)]
    fn the_name_column_stays_first(h: &mut Harness) {
        use ProcessColumn::*;
        start(h);
        let h = &*h;
        let mut page = mount(h);

        assert!(!enabled(&mut page, Name, ProcessesSettingsMark::Up));
        assert!(!enabled(&mut page, Name, ProcessesSettingsMark::Down));
        assert!(!enabled(&mut page, Pid, ProcessesSettingsMark::Up));
        assert!(enabled(&mut page, Pid, ProcessesSettingsMark::Down));
        assert!(enabled(&mut page, Disk, ProcessesSettingsMark::Down));
        let last = ProcessColumn::ALL[ProcessColumn::ALL.len() - 1];
        assert!(!enabled(&mut page, last, ProcessesSettingsMark::Down));
    }

    #[guinea::test(iterations = 4)]
    fn a_column_moved_down_trades_places_and_stays_there(h: &mut Harness) {
        use ProcessColumn::*;
        start(h);
        let h = &*h;
        let mut page = mount(h);

        page.within(Cpu).click(ProcessesSettingsMark::Down).settle();
        page.settle();

        let mut traded = ProcessColumn::ALL.to_vec();
        let cpu = traded.iter().position(|column| *column == Cpu).unwrap();
        traded.swap(cpu, cpu + 1);
        assert_eq!(traded[cpu], Memory);
        assert_eq!(columns(&page), names(&traded));
        let ranks = stored(h).columns().order();
        assert_eq!(ranks.get("memory"), Some(3));
        assert_eq!(ranks.get("cpu"), Some(4));
    }

    #[guinea::test(iterations = 4)]
    fn a_column_shown_here_is_kept_shown(h: &mut Harness) {
        use ProcessColumn::*;
        start(h);
        let h = &*h;
        let mut page = mount(h);

        page.within(Pid).click(ProcessesSettingsMark::Shown).settle();
        page.settle();

        assert_eq!(shown(&mut page, Pid), Some(PropertyValue::Bool(true)));
        let kept = stored(h).columns().configs().get(Pid.id());
        assert!(kept.is_some_and(|config| config.visible));

        page.within(Pid).click(ProcessesSettingsMark::Shown).settle();
        page.settle();

        assert_eq!(shown(&mut page, Pid), Some(PropertyValue::Bool(false)));
        let kept = stored(h).columns().configs().get(Pid.id());
        assert!(kept.is_some_and(|config| !config.visible));
    }

    #[guinea::test(iterations = 4)]
    fn the_columns_come_back_to_their_defaults(h: &mut Harness) {
        use ProcessColumn::*;
        start(h);
        let h = &*h;
        let mut page = mount(h);
        let reset = ProcessesSettingsMark::ResetColumns;
        assert!(!enabled(&mut page, reset, reset), "nothing to reset yet");

        page.within(Cpu).click(ProcessesSettingsMark::Down).settle();
        page.within(Pid).click(ProcessesSettingsMark::Shown).settle();
        page.settle();
        assert!(enabled(&mut page, reset, reset));

        page.click(reset).settle();
        page.settle();

        assert_eq!(columns(&page), names(&ProcessColumn::ALL));
        assert_eq!(shown(&mut page, Pid), Some(PropertyValue::Bool(false)));
        assert!(!enabled(&mut page, reset, reset));
        let kept = stored(h);
        assert_eq!(kept.columns().order().get("cpu"), None);
        assert!(kept.columns().configs().get(Pid.id()).is_some_and(|config| !config.visible));
    }

    #[guinea::test(iterations = 4)]
    fn a_section_moves_up_and_the_order_can_be_reset(h: &mut Harness) {
        start(h);
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
        let ranks = stored(h).grouping().section_order();
        assert_eq!(ranks.get(services.name()), at(services).map(|at| at as u32));

        page.click(ProcessesSettingsMark::ResetSections).settle();
        page.settle();

        assert_eq!(sections(&page), names(&all_sections()));
        let ranks = stored(h).grouping().section_order();
        assert_eq!(ranks.get(services.name()), None);
    }

    #[guinea::test(iterations = 4)]
    fn memory_is_shown_as_values_until_percents_are_chosen(h: &mut Harness) {
        start(h);
        let h = &*h;
        let mut page = mount(h);
        let choice = |page: &mut Mounted<'_, ProcessesSettings>| {
            let node = page.find(ProcessesSettingsMark::MemoryValues)?;
            page.property(node, PropertyId::ComboBoxSelectedIndex).cloned()
        };
        assert_eq!(choice(&mut page), Some(PropertyValue::SelectionIndex(Some(0))));

        page.send(ProcessesSettingsMsg::MemoryAsPercent(true));
        page.settle();

        assert_eq!(choice(&mut page), Some(PropertyValue::SelectionIndex(Some(1))));
        assert!(stored(h).columns().memory_as_percent().get());
    }

    #[guinea::test(iterations = 4)]
    fn the_breadcrumb_leads_back_to_the_processes(h: &mut Harness) {
        start(h);
        let h = &*h;
        let mut page = mount(h);

        page.click(ProcessesSettingsMark::Back).settle();
        page.settle();

        assert_eq!(page.navigated::<Route>(), [Route::Processes {}]);
    }
}
