use app_contracts::features::activity::ActivityState;
use app_contracts::features::window::PressedAway;
use guinea::feature::FeatureInitContext;
use guinea::prelude::GlobalEventBus;
use guinea::winui::{page, Page, PageCx, UpdateCx};
use ui::pages::activity::{ActivityPage, ActivityPageMsg};
use ui::theme::{scheme_context, Palette};
use windows_reactor::{Callback, View};

use crate::routes::Route;

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

    fn update(&mut self, message: ActivityPageMsg, cx: &mut UpdateCx<'_, Self>) {
        let (_, dispatch) = cx.read::<ActivityState>();
        self.0.update(message, &dispatch);
    }

    fn view(&self, cx: &mut PageCx<'_, '_, Self>) -> View {
        let (state, dispatch) = cx.read::<ActivityState>();
        let l10n = ui::l10n::use_tr(cx);
        let palette = Palette::of(cx.use_context(scheme_context()));
        let forward = cx.on(|message: ActivityPageMsg| message);
        let away = forward.clone();
        cx.use_effect_guard("uniproc::activity::menu_closes_on_press_away", (), move || {
            GlobalEventBus::subscribe_fn(move |_: PressedAway| {
                away.call(ActivityPageMsg::MenuDismiss);
            })
        });
        let nav = cx.navigate::<Route>();
        let manage = Callback::new(move |()| nav.to(Route::ActivityGroups {}));
        self.0.view(&state, &dispatch, &l10n, palette, forward, manage)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::sync::Arc;
    use std::time::Duration;

    use app_contracts::features::activity::{
        ActivityRow, Clock, DeleteGroup, DropRule, Filter, Group, Hue, MoveGroup, NewGroup, Pick, PutInGroup,
        RecolorGroup, RenameGroup, ShowGroup, ShowOther, ShowSeries, Span,
    };
    use app_contracts::features::agents::{
        ProcessCame, ProcessEvent, ProcessInstance, ProcessWent, WindowsProcessEvents,
    };
    use domain::features::activity::settings::{remember, remembered, remembered_groups, ActivitySettings};
    use guinea::prelude::Load;
    use domain::features::activity::{ActivityDeps, ActivityFeature};
    use guinea::app::Harness;
    use guinea::winui::harness::{Mounted, Node};
    use guinea_plugin_l10n::L10nPlugin;
    use guinea_plugin_store::{StoreAccess, StorePlugin};
    use ui::pages::activity::ActivityMark;

    use super::*;

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

    fn first_row<'a>(node: &'a Node) -> Option<&'a Node> {
        if node.id.as_deref() == Some("Row") {
            return Some(node);
        }
        node.children.iter().find_map(first_row)
    }

    #[guinea::test(iterations = 4)]
    fn a_clicked_row_is_marked_and_stays_where_it_was_open_when_newer_rows_come(h: &mut Harness) {
        start(h);
        let h = &*h;
        let mut page = mount(h);
        live(h, &mut page, vec![came(20, 10), came_from(21, 1, r"C:\Other\other.exe", 20)]);
        assert!(page.find(ActivityMark::Selected).is_none(), "{:#?}", page.tree());

        page.click(ActivityMark::Row).settle();
        page.settle();
        assert!(page.find(ActivityMark::Selected).is_some(), "{:#?}", page.tree());
        assert!(page.find(ActivityMark::Facts).is_some(), "{:#?}", page.tree());

        live(h, &mut page, vec![came_from(22, 1, r"C:\Third\third.exe", 30)]);

        let tree = page.tree();
        assert_eq!(rows(&tree), 3, "{tree:#?}");
        let top = first_row(&tree).expect("a row");
        assert!(says(top, "other.exe"), "the clicked row is still first: {tree:#?}");
        assert!(page.find(ActivityMark::Facts).is_some(), "{tree:#?}");
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

    fn right_click(page: &mut Mounted<'_, Activity>) {
        page.send(ActivityPageMsg::MenuAnchor { x: 40.0, y: 60.0 });
        page.click(ActivityMark::Row).settle();
        page.settle();
    }

    fn menu_open(page: &Mounted<'_, Activity>) -> bool {
        page.find(ActivityMark::RowMenu).is_some()
    }

    #[guinea::test(iterations = 4)]
    fn an_opened_row_shows_facts_and_leaves_hiding_to_its_menu(h: &mut Harness) {
        start(h);
        let h = &*h;
        let mut page = mount(h);
        live(h, &mut page, vec![came(20, 10)]);

        page.click(ActivityMark::Row).settle();
        page.settle();
        assert!(page.find(ActivityMark::Facts).is_some(), "{:#?}", page.tree());
        assert!(!menu_open(&page), "{:#?}", page.tree());
        assert!(page.find(ActivityMark::HideExe).is_none(), "{:#?}", page.tree());
        assert!(page.find(ActivityMark::Only).is_none(), "{:#?}", page.tree());
    }

    #[guinea::test(iterations = 4)]
    fn a_right_click_opens_the_menu_of_that_row_and_does_not_open_the_row(h: &mut Harness) {
        start(h);
        let h = &*h;
        let mut page = mount(h);
        live(h, &mut page, vec![came(20, 10)]);

        right_click(&mut page);
        assert!(menu_open(&page), "{:#?}", page.tree());
        assert!(page.find(ActivityMark::Facts).is_none(), "{:#?}", page.tree());
        assert!(page.find(ActivityMark::Selected).is_some(), "{:#?}", page.tree());
        assert!(page.find(ActivityMark::HideExe).is_some(), "{:#?}", page.tree());
        assert!(page.find(ActivityMark::HideFolder).is_some(), "{:#?}", page.tree());

        page.click(ActivityMark::RowMenuBackdrop).settle();
        page.settle();
        assert!(!menu_open(&page), "{:#?}", page.tree());

        right_click(&mut page);
        assert!(menu_open(&page), "{:#?}", page.tree());
        h.publish(PressedAway).settle();
        page.settle();
        assert!(!menu_open(&page), "a press elsewhere in the window closes it: {:#?}", page.tree());
    }

    #[guinea::test(iterations = 4)]
    fn a_series_menu_offers_to_hide_what_its_launcher_starts(h: &mut Harness) {
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

        right_click(&mut page);
        assert!(page.find(ActivityMark::HideLauncher).is_some(), "{:#?}", page.tree());
    }

    #[guinea::test(iterations = 4)]
    fn a_program_hidden_from_its_menu_leaves_the_list_and_no_chip_says_so(h: &mut Harness) {
        start(h);
        let h = &*h;
        let mut page = mount(h);
        live(h, &mut page, vec![came(20, 10), came_from(21, 1, r"C:\Other\other.exe", 20)]);
        right_click(&mut page);
        assert!(page.find(ActivityMark::HideExe).is_some(), "{:#?}", page.tree());

        page.click(ActivityMark::HideExe).settle();
        page.settle();
        assert!(!menu_open(&page), "a command closes the menu");
        h.advance(Duration::from_secs(1));
        page.settle();
        assert_eq!(rows(&page.tree()), 1, "{:#?}", page.tree());
        assert!(!says(&page.tree(), "other.exe"), "{:#?}", page.tree());
        assert!(page.find(ActivityMark::Picked).is_none(), "{:#?}", page.tree());
    }

    #[guinea::test(iterations = 4)]
    fn only_this_program_leaves_its_rows_alone_in_the_list(h: &mut Harness) {
        start(h);
        let h = &*h;
        let mut page = mount(h);
        live(h, &mut page, vec![came(20, 10), came_from(21, 1, r"C:\Other\other.exe", 20)]);
        right_click(&mut page);
        assert!(page.find(ActivityMark::Only).is_some(), "{:#?}", page.tree());

        page.click(ActivityMark::Only).settle();
        h.advance(Duration::from_secs(1));
        page.settle();
        assert_eq!(rows(&page.tree()), 1, "{:#?}", page.tree());
        assert!(!says(&page.tree(), "tool.exe"), "{:#?}", page.tree());
        assert!(page.find(ActivityMark::Picked).is_some(), "{:#?}", page.tree());
    }

    fn stored(h: &Harness) -> ActivitySettings {
        ActivitySettings::new_with(&h.segment().store().unwrap()).unwrap()
    }

    #[guinea::test(iterations = 4)]
    fn what_is_hidden_on_the_page_is_remembered_for_the_next_run(h: &mut Harness) {
        start(h);
        let h = &*h;
        let mut page = mount(h);
        live(h, &mut page, vec![came(20, 10), came_from(21, 1, r"C:\Other\other.exe", 20)]);
        right_click(&mut page);
        page.click(ActivityMark::HideExe).settle();
        page.settle();
        page.click(ActivityMark::Went).settle();
        page.settle();

        let shown = h.state::<ActivityState>().filter.clone();
        assert!(!shown.went && shown.hidden.len() == 1, "{shown:#?}");
        assert_eq!(remembered(&stored(h)), (Span::default(), shown));
    }

    #[guinea::test(iterations = 4)]
    fn the_page_starts_with_what_was_chosen_last_run(h: &mut Harness) {
        h.plugin(StorePlugin::in_memory())
            .unwrap()
            .plugin(L10nPlugin::<app_contracts::l10n::L10n>::new("en"))
            .unwrap()
            .provide(ActivityDeps {
                now: || BASE + HOUR - SECOND,
                clock: utc,
            });
        let chosen = Filter {
            went: false,
            hidden: vec![Pick::Folder(r"c:\other".into())],
            ..Filter::default()
        };
        remember(&stored(h), Span::Quarter, &chosen).unwrap();

        h.feature(ActivityFeature).unwrap();

        let state = h.state::<ActivityState>();
        assert_eq!((state.span, &state.filter), (Span::Quarter, &chosen));
    }

    #[guinea::test(iterations = 4)]
    fn every_span_is_offered_and_a_picked_one_is_shown_and_remembered(h: &mut Harness) {
        start(h);
        let h = &*h;
        let mut page = mount(h);
        live(h, &mut page, vec![came(20, 10)]);

        let spans = [
            Span::HalfMinute,
            Span::FiveMinutes,
            Span::Quarter,
            Span::HalfHour,
            Span::Hour,
            Span::Day,
            Span::Connected,
        ];
        let offered: Vec<Span> = spans.into_iter().filter(|&span| page.find(span).is_some()).collect();
        assert_eq!(offered, spans, "{:#?}", page.tree());

        page.click(Span::Day).settle();
        page.settle();
        assert_eq!(h.state::<ActivityState>().span, Span::Day);
        assert_eq!(remembered(&stored(h)).0, Span::Day);
    }

    const TASKHOST: &str = r"C:\Windows\System32\taskhostw.exe";

    fn painted(h: &Harness) -> Vec<(String, Option<Hue>)> {
        let state = h.state::<ActivityState>();
        let Load::Ready(view) = &state.view else {
            return Vec::new();
        };
        let mut rows: Vec<(String, Option<Hue>)> = view
            .rows
            .iter()
            .map(|row| match row {
                ActivityRow::Came(came) => (came.name.to_string(), came.hue),
                other => (format!("{other:?}"), other.hue()),
            })
            .collect();
        rows.sort();
        rows
    }

    fn act(h: &Harness, action: impl Sized + 'static) {
        h.act::<ActivityState>(action).settle();
        h.advance(Duration::from_secs(1));
    }

    fn tool_and_taskhost(h: &Harness) -> Mounted<'_, Activity> {
        let mut page = mount(h);
        act(h, ShowSeries(false));
        live(h, &mut page, vec![came(20, 10), came_from(21, 1, TASKHOST, 20)]);
        page
    }

    #[guinea::test(iterations = 4)]
    fn the_windows_background_group_is_there_from_the_start_and_paints_taskhostw(h: &mut Harness) {
        start(h);
        let h = &*h;
        let _page = tool_and_taskhost(h);

        assert_eq!(h.state::<ActivityState>().groups, [Group::windows_background()]);
        assert_eq!(
            painted(h),
            [("taskhostw.exe".into(), Some(Hue::Teal)), ("tool.exe".into(), None)]
        );
    }

    #[guinea::test(iterations = 4)]
    fn a_group_switched_off_leaves_the_page_and_stays_off_for_the_next_run(h: &mut Harness) {
        start(h);
        let h = &*h;
        let _page = tool_and_taskhost(h);

        act(
            h,
            ShowGroup {
                group: Group::WINDOWS_BACKGROUND.into(),
                shown: false,
            },
        );

        assert_eq!(painted(h), [("tool.exe".into(), None)]);
        assert!(!remembered_groups(&stored(h))[0].shown);
    }

    #[guinea::test(iterations = 4)]
    fn the_rest_switched_off_leaves_only_grouped_processes_and_is_remembered(h: &mut Harness) {
        start(h);
        let h = &*h;
        let _page = tool_and_taskhost(h);

        act(h, ShowOther(false));

        assert_eq!(painted(h), [("taskhostw.exe".into(), Some(Hue::Teal))]);
        assert!(!remembered(&stored(h)).1.other);
    }

    #[guinea::test(iterations = 4)]
    fn a_program_put_in_a_new_group_leaves_the_group_it_was_in(h: &mut Harness) {
        start(h);
        let h = &*h;
        let _page = tool_and_taskhost(h);
        let tool = Pick::Exe("tool.exe".into());

        act(
            h,
            PutInGroup {
                group: Group::WINDOWS_BACKGROUND.into(),
                rule: tool.clone(),
            },
        );
        assert_eq!(painted(h)[1], ("tool.exe".into(), Some(Hue::Teal)));

        act(h, NewGroup(tool.clone()));

        let groups = h.state::<ActivityState>().groups.clone();
        assert_eq!(groups.len(), 2, "{groups:#?}");
        let new = &groups[1];
        assert_eq!((new.name.as_str(), &new.rules[..]), ("tool.exe", std::slice::from_ref(&tool)));
        assert_ne!(new.hue, Hue::Teal);
        assert!(!groups[0].rules.contains(&tool), "{groups:#?}");
        assert_eq!(painted(h)[1], ("tool.exe".into(), Some(new.hue)));
        assert_eq!(remembered_groups(&stored(h)), groups);
    }

    #[guinea::test(iterations = 4)]
    fn a_group_is_renamed_recoloured_moved_emptied_and_deleted_but_the_built_in_one_stays(h: &mut Harness) {
        start(h);
        let h = &*h;
        let _page = tool_and_taskhost(h);
        let tool = Pick::Exe("tool.exe".into());
        act(h, NewGroup(tool.clone()));
        let groups = h.state::<ActivityState>().groups.clone();
        assert_eq!(groups.len(), 2, "{groups:#?}");
        let id = groups[1].id.clone();

        act(
            h,
            RenameGroup {
                group: id.clone(),
                name: "Dev tooling".into(),
            },
        );
        act(
            h,
            RecolorGroup {
                group: id.clone(),
                hue: Hue::Coral,
            },
        );
        act(
            h,
            MoveGroup {
                group: id.clone(),
                up: true,
            },
        );
        let groups = h.state::<ActivityState>().groups.clone();
        assert_eq!(
            (groups[0].id.as_str(), groups[0].name.as_str(), groups[0].hue),
            (id.as_str(), "Dev tooling", Hue::Coral)
        );

        act(
            h,
            DropRule {
                group: id.clone(),
                rule: tool,
            },
        );
        assert!(h.state::<ActivityState>().groups[0].rules.is_empty());

        act(h, DeleteGroup(Group::WINDOWS_BACKGROUND.into()));
        act(h, DeleteGroup(id));
        assert_eq!(h.state::<ActivityState>().groups, [Group::windows_background()]);
        assert_eq!(remembered_groups(&stored(h)), [Group::windows_background()]);
    }

    fn marked(node: &Node, mark: &str) -> usize {
        usize::from(node.id.as_deref() == Some(mark)) + node.children.iter().map(|child| marked(child, mark)).sum::<usize>()
    }

    #[guinea::test(iterations = 4)]
    fn the_legend_switches_a_group_and_the_rest_off_and_on(h: &mut Harness) {
        start(h);
        let h = &*h;
        let mut page = tool_and_taskhost(h);
        page.settle();
        assert!(page.find(ActivityMark::LegendGroup).is_some(), "{:#?}", page.tree());
        assert!(page.find(ActivityMark::LegendOther).is_some(), "{:#?}", page.tree());

        page.click(ActivityMark::LegendGroup).settle();
        page.settle();
        assert!(!h.state::<ActivityState>().groups[0].shown);
        page.click(ActivityMark::LegendGroup).settle();
        page.settle();
        assert!(h.state::<ActivityState>().groups[0].shown);

        page.click(ActivityMark::LegendOther).settle();
        page.settle();
        assert!(!h.state::<ActivityState>().filter.other);
    }

    #[guinea::test(iterations = 4)]
    fn a_row_in_a_group_has_a_stripe_and_one_in_none_has_not(h: &mut Harness) {
        start(h);
        let h = &*h;
        let mut page = tool_and_taskhost(h);
        page.settle();

        let tree = page.tree();
        assert_eq!(rows(&tree), 2, "{tree:#?}");
        assert_eq!(marked(&tree, "Stripe"), 1, "{tree:#?}");
    }

    #[guinea::test(iterations = 4)]
    fn the_row_menu_puts_a_program_in_a_new_group_and_back_in_one_there_is(h: &mut Harness) {
        start(h);
        let h = &*h;
        let mut page = tool_and_taskhost(h);
        page.settle();
        let taskhost = Pick::Exe("taskhostw.exe".into());

        right_click(&mut page);
        assert!(page.find(ActivityMark::NewGroup).is_some(), "{:#?}", page.tree());
        page.click(ActivityMark::NewGroup).settle();
        page.settle();
        let groups = h.state::<ActivityState>().groups.clone();
        assert_eq!(groups.len(), 2, "{groups:#?}");
        assert_eq!(groups[1].rules, std::slice::from_ref(&taskhost), "{groups:#?}");
        assert!(!menu_open(&page), "a command closes the menu");

        right_click(&mut page);
        assert!(page.find(ActivityMark::PutIn).is_some(), "{:#?}", page.tree());
        page.click(ActivityMark::PutIn).settle();
        page.settle();
        let groups = h.state::<ActivityState>().groups.clone();
        assert_eq!(groups[0].rules, std::slice::from_ref(&taskhost), "{groups:#?}");
        assert!(groups[1].rules.is_empty(), "{groups:#?}");
    }

    #[guinea::test(iterations = 4)]
    fn the_menu_opens_the_page_that_manages_groups(h: &mut Harness) {
        start(h);
        let h = &*h;
        let mut page = mount(h);
        live(h, &mut page, vec![came(20, 10)]);
        assert!(page.find(ActivityMark::ManageGroups).is_some(), "{:#?}", page.tree());

        page.click(ActivityMark::ManageGroups).settle();

        assert_eq!(page.navigated::<Route>(), [Route::ActivityGroups {}]);
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
