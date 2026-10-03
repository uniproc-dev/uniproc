use app_contracts::features::activity::ActivityState;
use guinea::feature::FeatureInitContext;
use guinea::winui::{page, Page, PageCx, UpdateCx};
use ui::pages::activity::{ActivityPage, ActivityPageMsg};
use ui::theme::{scheme_context, Palette};
use windows_reactor::View;

#[derive(Default)]
pub struct Activity(ActivityPage);

#[page]
impl Page for Activity {
    type Params = crate::routes::ActivityParams;
    type Installs = ();
    type Message = ActivityPageMsg;

    fn install(_ctx: &FeatureInitContext, _params: &Self::Params) -> anyhow::Result<Self::Installs> {
        Ok(())
    }

    fn update(&mut self, message: ActivityPageMsg, _cx: &mut UpdateCx<'_, Self>) {
        self.0.update(message);
    }

    fn view(&self, cx: &mut PageCx<'_, Self>) -> View {
        let (state, dispatch) = cx.read::<ActivityState, _>();
        let l10n = ui::l10n::use_tr(cx);
        let palette = Palette::of(cx.use_context(scheme_context()));
        let forward = cx.on(|message: ActivityPageMsg| message);
        self.0.view(&state, &dispatch, &l10n, palette, forward)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::sync::Arc;
    use std::time::Duration;

    use app_contracts::features::activity::Clock;
    use app_contracts::features::agents::{
        ProcessCame, ProcessEvent, ProcessInstance, ProcessWent, WindowsProcessEvents,
    };
    use domain::features::activity::{ActivityDeps, ActivityFeature};
    use guinea::app::Harness;
    use guinea::winui::harness::{Drag, Mounted, Node};
    use guinea_plugin_l10n::L10nPlugin;
    use guinea_plugin_store::StorePlugin;
    use ui::pages::activity::ActivityMark;

    use super::*;
    use crate::routes::Route;

    const SECOND: u64 = 10_000_000;
    const HOUR: u64 = 3600 * SECOND;
    const BASE: u64 = 1000 * HOUR;

    fn utc(at: u64) -> Clock {
        let seconds = at / SECOND;
        Clock {
            hour: (seconds / 3600 % 24) as u8,
            minute: (seconds / 60 % 60) as u8,
            second: (seconds % 60) as u8,
        }
    }

    fn start(h: &mut Harness) {
        start_at(h, || BASE + HOUR - SECOND);
    }

    fn start_at(h: &mut Harness, now: fn() -> u64) {
        h.plugin(StorePlugin::in_memory())
            .unwrap()
            .plugin(L10nPlugin::<app_contracts::l10n::L10n>::new("en"))
            .unwrap()
            .provide(ActivityDeps { now, clock: utc });
        h.feature(ActivityFeature).unwrap();
    }

    thread_local! {
        static DRAWN: Cell<usize> = const { Cell::new(0) };
    }

    fn counted_now() -> u64 {
        DRAWN.with(|drawn| drawn.set(drawn.get() + 1));
        BASE + HOUR - SECOND
    }

    fn mount(h: &Harness) -> Mounted<'_, Activity> {
        let mut page =
            Mounted::mount_at(h.child(), crate::routes::ActivityParams::default(), Route::Activity {}).unwrap();
        page.settle();
        page
    }

    fn id(pid: u32) -> ProcessInstance {
        ProcessInstance { pid, sequence: 7 }
    }

    fn came(pid: u32, minute: u64) -> ProcessEvent {
        ProcessEvent::Came(ProcessCame {
            instance: id(pid),
            parent: id(1),
            at: BASE + minute * 60 * SECOND,
            image_path: r"C:\Tools\tool.exe".into(),
            command_line: "tool.exe --check".into(),
            ..Default::default()
        })
    }

    fn went(pid: u32, minute: u64) -> ProcessEvent {
        ProcessEvent::Went(ProcessWent {
            instance: id(pid),
            at: BASE + minute * 60 * SECOND,
            ..Default::default()
        })
    }

    fn says(node: &Node, wanted: &str) -> bool {
        node.text.as_deref().map(|text| text.replace(['\u{2068}', '\u{2069}'], "")).as_deref() == Some(wanted)
            || node.children.iter().any(|child| says(child, wanted))
    }

    fn tell(h: &Harness, events: Vec<ProcessEvent>) {
        h.publish(WindowsProcessEvents {
            events: Arc::from(events),
            ..WindowsProcessEvents::default()
        })
        .settle();
    }

    fn mentions(node: &Node, part: &str) -> bool {
        node.text.as_deref().is_some_and(|text| text.replace(['\u{2068}', '\u{2069}'], "").contains(part))
            || node.children.iter().any(|child| mentions(child, part))
    }

    fn live(h: &Harness, page: &mut Mounted<'_, Activity>, events: Vec<ProcessEvent>) {
        tell(h, events);
        h.advance(Duration::from_secs(1));
        page.settle();
    }

    fn rows(node: &Node) -> usize {
        usize::from(node.id.as_deref() == Some("Row")) + node.children.iter().map(rows).sum::<usize>()
    }

    #[guinea::test(iterations = 4)]
    fn a_process_that_came_is_listed_by_name_and_its_command_line_waits_in_its_facts(h: &mut Harness) {
        start(h);
        let h = &*h;
        let mut page = mount(h);
        live(h, &mut page, vec![came(20, 10)]);

        let tree = page.tree();
        assert!(says(&tree, "tool.exe"), "{tree:#?}");
        assert!(!says(&tree, "tool.exe --check"), "{tree:#?}");
    }

    #[guinea::test(iterations = 4)]
    fn a_flood_of_batches_is_drawn_once_and_still_within_a_second(h: &mut Harness) {
        start_at(h, counted_now);
        let h = &*h;
        let mut page = mount(h);
        DRAWN.with(|drawn| drawn.set(0));

        for pid in 100..140 {
            tell(h, vec![came(pid, 10)]);
        }
        h.advance(Duration::from_secs(1));
        page.settle();

        assert!(DRAWN.with(Cell::get) <= 2, "drawn {} times", DRAWN.with(Cell::get));
        assert_eq!(rows(&page.tree()), 40, "{:#?}", page.tree());
    }

    #[guinea::test(iterations = 4)]
    fn how_long_a_process_lived_is_one_unit_picked_by_its_size(h: &mut Harness) {
        start(h);
        let h = &*h;
        let mut page = mount(h);
        let quick = ProcessEvent::Went(ProcessWent {
            instance: id(20),
            at: BASE + 10 * 60 * SECOND + 340 * SECOND / 1000,
            ..Default::default()
        });
        live(h, &mut page, vec![came(20, 10), quick, came(21, 20), went(21, 26)]);

        let tree = page.tree();
        assert!(says(&tree, "340 ms"), "{tree:#?}");
        assert!(says(&tree, "6 min"), "{tree:#?}");
    }

    #[guinea::test(iterations = 4)]
    fn an_area_drawn_on_the_chart_narrows_the_list_and_a_click_brings_it_back(h: &mut Harness) {
        start(h);
        let h = &*h;
        let mut page = mount(h);
        live(h, &mut page, vec![came(20, 10), came(21, 40)]);
        assert_eq!(rows(&page.tree()), 2, "{:#?}", page.tree());

        page.drag(ActivityMark::Scatter, Drag::by(400.0, 120.0).from(0.0, 2.0)).settle();
        h.advance(Duration::from_secs(1));
        page.settle();
        assert_eq!(rows(&page.tree()), 1, "{:#?}", page.tree());
        assert!(page.find(ActivityMark::ClearArea).is_some(), "{:#?}", page.tree());

        page.drag(ActivityMark::Scatter, Drag::by(1.0, 1.0).from(600.0, 60.0)).settle();
        h.advance(Duration::from_secs(1));
        page.settle();
        assert_eq!(rows(&page.tree()), 2, "{:#?}", page.tree());
    }

    #[guinea::test(iterations = 4)]
    fn the_lifetime_scale_is_labelled_with_durations_only(h: &mut Harness) {
        start(h);
        let h = &*h;
        let mut page = mount(h);
        live(h, &mut page, vec![came(20, 10), went(30, 20)]);

        let tree = page.tree();
        assert!(says(&tree, "1 min") && says(&tree, "100 ms"), "{tree:#?}");
        assert!(!says(&tree, "running") && !says(&tree, "before"), "{tree:#?}");
    }

    #[guinea::test(iterations = 4)]
    fn an_exit_code_waits_in_the_opened_row(h: &mut Harness) {
        start(h);
        let h = &*h;
        let mut page = mount(h);
        let coded = ProcessEvent::Went(ProcessWent {
            instance: id(30),
            at: BASE + 20 * 60 * SECOND,
            exit_code: 3,
            image_name: "tool.exe".into(),
            ..Default::default()
        });
        live(h, &mut page, vec![coded]);
        let shows_code = |node: &Node| mentions(node, "code 3");
        assert!(!shows_code(&page.tree()), "{:#?}", page.tree());

        page.click(ActivityMark::Row).settle();
        page.settle();
        assert!(shows_code(&page.tree()), "{:#?}", page.tree());
    }

    #[guinea::test(iterations = 4)]
    fn a_row_opens_to_its_facts(h: &mut Harness) {
        start(h);
        let h = &*h;
        let mut page = mount(h);
        live(h, &mut page, vec![came(20, 10)]);
        assert_eq!(rows(&page.tree()), 1, "{:#?}", page.tree());
        assert!(page.find(ActivityMark::Facts).is_none());

        page.click(ActivityMark::Row).settle();
        page.settle();

        let tree = page.tree();
        assert!(page.find(ActivityMark::Facts).is_some(), "{tree:#?}");
        assert!(says(&tree, "tool.exe --check"), "{tree:#?}");
    }

    #[guinea::test(iterations = 4)]
    fn an_exit_of_a_process_never_seen_says_so_instead_of_a_bare_number(h: &mut Harness) {
        start(h);
        let h = &*h;
        let mut page = mount(h);
        live(h, &mut page, vec![went(30, 20)]);

        let tree = page.tree();
        assert!(says(&tree, "Process 30"), "{tree:#?}");
        assert!(!says(&tree, "30"), "{tree:#?}");
    }

    fn came_from(pid: u32, parent: u32, path: &str, minute: u64) -> ProcessEvent {
        ProcessEvent::Came(ProcessCame {
            instance: id(pid),
            parent: id(parent),
            at: BASE + minute * 60 * SECOND,
            image_path: path.into(),
            command_line: path.into(),
            ..Default::default()
        })
    }

    const GIT: &str = r"C:\Program Files\Git\cmd\git.exe";

    #[guinea::test(iterations = 4)]
    fn repeats_from_one_launcher_and_folder_are_one_row_until_grouping_is_off(h: &mut Harness) {
        start(h);
        let h = &*h;
        let mut page = mount(h);
        live(
            h,
            &mut page,
            vec![
                came_from(4, 1, r"C:\Tools\claude.exe", 5),
                came_from(20, 4, GIT, 10),
                came_from(21, 4, GIT, 20),
                came_from(22, 4, GIT, 30),
            ],
        );
        assert_eq!(rows(&page.tree()), 2, "{:#?}", page.tree());
        assert!(says(&page.tree(), "git.exe × 3"), "{:#?}", page.tree());

        page.click(ActivityMark::Series).settle();
        h.advance(Duration::from_secs(1));
        page.settle();
        assert_eq!(rows(&page.tree()), 4, "{:#?}", page.tree());
    }

    #[guinea::test(iterations = 4)]
    fn a_program_hidden_from_its_opened_row_leaves_the_list_until_its_chip_is_clicked(h: &mut Harness) {
        start(h);
        let h = &*h;
        let mut page = mount(h);
        live(h, &mut page, vec![came(20, 10), came_from(21, 1, r"C:\Other\other.exe", 20)]);
        page.click(ActivityMark::Row).settle();
        page.settle();
        assert!(page.find(ActivityMark::HideExe).is_some(), "{:#?}", page.tree());

        page.click(ActivityMark::HideExe).settle();
        h.advance(Duration::from_secs(1));
        page.settle();
        assert_eq!(rows(&page.tree()), 1, "{:#?}", page.tree());
        assert!(!says(&page.tree(), "other.exe"), "{:#?}", page.tree());
        assert!(page.find(ActivityMark::Picked).is_some(), "{:#?}", page.tree());

        page.click(ActivityMark::Picked).settle();
        h.advance(Duration::from_secs(1));
        page.settle();
        assert_eq!(rows(&page.tree()), 2, "{:#?}", page.tree());
        assert!(page.find(ActivityMark::Picked).is_none(), "{:#?}", page.tree());
    }

    #[guinea::test(iterations = 4)]
    fn only_this_program_leaves_its_rows_alone_in_the_list(h: &mut Harness) {
        start(h);
        let h = &*h;
        let mut page = mount(h);
        live(h, &mut page, vec![came(20, 10), came_from(21, 1, r"C:\Other\other.exe", 20)]);
        page.click(ActivityMark::Row).settle();
        page.settle();
        assert!(page.find(ActivityMark::Only).is_some(), "{:#?}", page.tree());

        page.click(ActivityMark::Only).settle();
        h.advance(Duration::from_secs(1));
        page.settle();
        assert_eq!(rows(&page.tree()), 1, "{:#?}", page.tree());
        assert!(!says(&page.tree(), "tool.exe"), "{:#?}", page.tree());
        assert!(page.find(ActivityMark::Picked).is_some(), "{:#?}", page.tree());
    }

    #[guinea::test(iterations = 4)]
    fn hiding_exits_from_the_menu_leaves_nothing_when_only_exits_happened(h: &mut Harness) {
        start(h);
        let h = &*h;
        let mut page = mount(h);
        live(h, &mut page, vec![went(30, 20)]);
        assert_eq!(rows(&page.tree()), 1, "{:#?}", page.tree());
        assert!(page.find(ActivityMark::Menu).is_some(), "{:#?}", page.tree());

        page.click(ActivityMark::Went).settle();
        page.settle();

        assert!(page.find(ActivityMark::Empty).is_some(), "{:#?}", page.tree());
    }
}
