use app_contracts::features::processes::ProcessesState;
use app_contracts::features::window::PressedAway;
use domain::features::processes::settings::ProcessesSettings;
use domain::features::processes::{ProcessesDeps, ProcessesFeature};
use guinea::feature::FeatureInitContext;
use guinea::prelude::GlobalEventBus;
use guinea::winui::{page, Page, PageCx, UpdateCx};
use ui::pages::processes::{ProcessesMsg, ProcessesPage, ProcessesSettingsMaps};
use ui::theme::{scheme_context, Palette};
use windows_reactor::View;

#[derive(Default)]
pub struct Processes(ProcessesPage);

fn open_settings() -> Option<ProcessesSettingsMaps> {
    let settings = ProcessesSettings::new()
        .inspect_err(|err| tracing::error!(?err, "processes settings did not open"))
        .ok()?;
    Some(ProcessesSettingsMaps {
        columns: settings.columns().configs().clone(),
        column_order: settings.columns().order().clone(),
        collapsed_sections: settings.grouping().collapsed_sections().clone(),
        pins: settings.grouping().pins().clone(),
        group_by_type: settings.grouping().by_type().clone(),
        section_order: settings.grouping().section_order().clone(),
    })
}

#[page]
impl Page for Processes {
    const CACHE_STATE_IN_MEMORY: bool = true;

    type Params = crate::routes::ProcessesParams;
    type Installs = ProcessesFeature;
    type Message = ProcessesMsg;

    fn install(ctx: &FeatureInitContext, _params: &Self::Params) -> anyhow::Result<Self::Installs> {
        ctx.install(&ctx.require_or_default::<ProcessesDeps>())
    }

    fn init(_ctx: &FeatureInitContext, _params: &Self::Params) -> Self {
        Self(ProcessesPage::new(open_settings()))
    }

    fn update(&mut self, message: ProcessesMsg, _cx: &mut UpdateCx<'_, Self>) {
        self.0.update(message);
    }

    fn view(&self, cx: &mut PageCx<'_, Self>) -> View {
        let (state, dispatch) = cx.use_reducer::<ProcessesState, _>();
        let l10n = ui::l10n::use_tr(cx);
        let palette = Palette::of(cx.use_context(scheme_context()));
        let forward = cx.on(|message: ProcessesMsg| message);
        let away = forward.clone();
        cx.use_effect_guard("uniproc::processes::menu_closes_on_press_away", (), move || {
            GlobalEventBus::subscribe_fn(move |_: PressedAway| {
                let _ = away.call(ProcessesMsg::MenuDismiss);
            })
        });
        let dismiss = forward.clone();
        let selected = state.selected;
        cx.use_effect("uniproc::processes::menu_needs_a_selection", selected, move || {
            if selected.is_none() {
                let _ = dismiss.call(ProcessesMsg::MenuDismiss);
            }
            None
        });
        self.0.view(&state, &dispatch, &l10n, palette, forward)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use app_contracts::features::agents::{
        AgentConnectionState, EnvironmentKind, LinuxEnvironmentInfo, LinuxProcessStats, RemoteScan,
        RemoteScanResult, SignatureStatus, WindowsActionRequest, WindowsProcessStats, WindowsReport,
        WindowsReportMessage, WindowsServiceState, WindowsServiceStats,
    };
    use std::time::Duration;

    use app_contracts::features::agent_link::AgentLinkState;
    use std::cell::Cell;
    use std::rc::Rc;

    use app_contracts::features::processes::{
        Deselect, PinnedProcess, ProcessColumn, ProcessCommand, ProcessesState, RunProcessCommand,
        Sort, Terminate, WindowCommand,
    };
    use guinea::prelude::GlobalEventBus;
    use domain::features::agent_link::{AgentLinkDeps, AgentLinkFeature};
    use domain::features::processes::shell::ShellRequest;
    use domain::features::processes::windows_scan::AppWindows;
    use guinea::app::Harness;
    use guinea::winui::harness::{Mounted, Node, PropertyId, PropertyValue};
    use guinea_plugin_l10n::L10nPlugin;
    use guinea_plugin_store::amethystate::store::builder::Backend;
    use guinea_plugin_store::StorePlugin;
    use app_contracts::features::processes::ProcessCategory;
    use guinea::winui::harness::Drag;
    use ui::pages::processes::ProcessesMark;
    use ui::widgets::page::PageMark;
    use ui::widgets::selection::SelectionMark;
    use app_contracts::features::window::PressedAway;
    use windows_reactor::ColorScheme;

    use super::*;

    const NOTEPAD: u32 = 10;
    const NOTEPAD_PATH: &str = r"C:\Windows\notepad.exe";
    const CHROME: [u32; 3] = [21, 22, 23];
    const SVCHOST: u32 = 30;
    const CMD: u32 = 40;
    const CONHOST: u32 = 41;
    const IDLE: u32 = 50;
    const BROKER: u32 = 60;

    thread_local! {
        static NOTEPAD_WINDOW: std::cell::Cell<bool> = const { std::cell::Cell::new(true) };
        static SHELL: std::cell::RefCell<Vec<ShellRequest>> = const { std::cell::RefCell::new(Vec::new()) };
    }

    fn fake_shell(request: ShellRequest) {
        SHELL.with_borrow_mut(|requests| requests.push(request));
    }

    fn shell_requests() -> Vec<ShellRequest> {
        SHELL.with_borrow(Clone::clone)
    }

    fn desktop_windows() -> AppWindows {
        let mut windows = AppWindows::default();
        if NOTEPAD_WINDOW.get() {
            windows.add(NOTEPAD, 0, "Untitled - Notepad".to_string());
        }
        windows
    }

    fn process(pid: u32, name: &str, cpu: f32) -> WindowsProcessStats {
        WindowsProcessStats {
            pid,
            name: name.into(),
            cpu_percent: cpu,
            private_working_set_bytes: 1_024_000 * u64::from(pid),
            signature: SignatureStatus::ThirdParty,
            ..Default::default()
        }
    }

    fn machine() -> Vec<WindowsProcessStats> {
        let mut svchost = process(SVCHOST, "svchost.exe", 0.5);
        svchost.is_service = true;
        svchost.signature = SignatureStatus::Microsoft;
        let mut cmd = process(CMD, "cmd.exe", 0.2);
        cmd.console_host_pid = CONHOST;
        let mut conhost = process(CONHOST, "conhost.exe", 0.1);
        conhost.parent_pid = CMD;
        let mut broker = process(BROKER, "RuntimeBroker.exe", 0.3);
        broker.signature = SignatureStatus::Microsoft;

        let mut notepad = process(NOTEPAD, "notepad.exe", 5.0);
        notepad.image_path = NOTEPAD_PATH.into();

        vec![
            broker,
            notepad,
            process(CHROME[0], "chrome.exe", 3.0),
            process(CHROME[1], "chrome.exe", 2.0),
            process(CHROME[2], "chrome.exe", 1.0),
            svchost,
            cmd,
            conhost,
            process(IDLE, "idle.exe", 0.0),
        ]
    }

    fn without(gone: &[u32]) -> Vec<WindowsProcessStats> {
        machine()
            .into_iter()
            .filter(|p| !gone.contains(&p.pid))
            .collect()
    }

    fn service(name: &str, display_name: &str) -> WindowsServiceStats {
        WindowsServiceStats {
            name: name.into(),
            display_name: display_name.into(),
            pid: SVCHOST,
            state: WindowsServiceState::Running,
            ..Default::default()
        }
    }

    fn report(h: &Harness, processes: Vec<WindowsProcessStats>) {
        let report = WindowsReport {
            processes,
            services: vec![
                service("Audiosrv", "Windows Audio"),
                service("Dhcp", "DHCP Client"),
            ],
            ..Default::default()
        };
        h.publish(WindowsReportMessage::Report(Arc::new(report)))
            .settle();
    }

    fn start(h: &mut Harness) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        NOTEPAD_WINDOW.set(true);
        SHELL.with_borrow_mut(Vec::clear);
        h.plugin(StorePlugin::at(dir.path().join("settings")).backend(Backend::Json))
            .unwrap()
            .plugin(L10nPlugin::<app_contracts::l10n::L10n>::new("en"))
            .unwrap()
            .provide(ProcessesDeps {
                windows: desktop_windows,
                shell: fake_shell,
            });
        dir
    }

    fn mount(h: &Harness) -> Mounted<'_, Processes> {
        let params = crate::routes::ProcessesParams::default();
        let mut page = Mounted::mount_with(&h.segment(), params, |page| {
            View::provide(scheme_context(), ColorScheme::Dark, page)
        })
        .unwrap();
        report(h, machine());
        page.settle();
        page
    }

    fn texts(node: &Node, out: &mut Vec<String>) {
        if let Some(text) = &node.text {
            out.push(text.clone());
        }
        for child in &node.children {
            texts(child, out);
        }
    }

    fn cell(item: &Node, column: ProcessColumn) -> String {
        let mut out = Vec::new();
        if let Some(cell) = item.find(column) {
            texts(cell, &mut out);
        }
        out.join(" ")
    }

    fn label(item: &Node) -> String {
        cell(item, ProcessColumn::Name)
    }

    fn labels(page: &mut Mounted<'_, Processes>) -> Vec<String> {
        page.items()
            .iter()
            .map(label)
            .filter(|label| !label.is_empty())
            .collect()
    }

    fn position(page: &mut Mounted<'_, Processes>, wanted: &str) -> Option<usize> {
        labels(page).iter().position(|label| label == wanted)
    }

    fn select(page: &mut Mounted<'_, Processes>, wanted: &str) {
        page.item_where(|item| label(item) == wanted).click_here();
        page.settle();
    }

    fn open(page: &mut Mounted<'_, Processes>, wanted: &str) {
        page.item_where(|item| label(item) == wanted)
            .click(ProcessesMark::Chevron);
        page.settle();
    }

    fn end_task_enabled(page: &Mounted<'_, Processes>) -> bool {
        let button = page.find(ProcessesMark::EndTask).expect("End task is on the page");
        page.property(button, PropertyId::ButtonIsEnabled) != Some(&PropertyValue::Bool(false))
    }

    fn is_marked_selected(item: &Node) -> bool {
        item.find(ProcessColumn::Name)
            .and_then(|cell| cell.children.first())
            .is_some_and(|grid| grid.children.len() == 2)
    }

    fn plate(page: &mut Mounted<'_, Processes>, index: usize) -> Option<PropertyValue> {
        let row = page.item(index).tree();
        let plate = row.children.first()?.children.first()?.at;
        page.at(plate).property(PropertyId::BorderBackground).cloned()
    }

    fn painted(page: &mut Mounted<'_, Processes>) -> Vec<String> {
        let items = page.items();
        let heading = items
            .iter()
            .position(|item| label(item).starts_with("Background processes"))
            .unwrap();
        let unpainted = plate(page, heading);
        (0..items.len())
            .filter(|at| !label(&items[*at]).is_empty())
            .filter(|at| plate(page, *at) != unpainted)
            .map(|at| label(&items[at]))
            .collect()
    }

    fn chevron_takes_the_whole_slot(page: &mut Mounted<'_, Processes>, wanted: &str) -> bool {
        let chevron = page
            .item_where(|item| label(item) == wanted)
            .find(ProcessesMark::Chevron)
            .expect("the row has a chevron");
        page.property(chevron, PropertyId::BorderBackground).is_some()
    }

    fn marked_selected(page: &mut Mounted<'_, Processes>) -> Vec<String> {
        page.items()
            .iter()
            .filter(|item| is_marked_selected(item))
            .map(label)
            .collect()
    }

    fn headings(page: &mut Mounted<'_, Processes>) -> Vec<String> {
        labels(page)
            .into_iter()
            .filter(|label| label.starts_with("Apps") || label.starts_with("Background processes"))
            .collect()
    }

    fn toggle_group_by_type(page: &mut Mounted<'_, Processes>) {
        page.click(ProcessesMark::GroupByType).settle();
        page.settle();
    }

    #[guinea::test(iterations = 8, exclusive = "store")]
    fn the_group_by_type_toggle_drops_the_sections_and_brings_them_back(h: &mut Harness) {
        let _store = start(h);
        let h = &*h;
        let mut page = mount(h);
        assert!(!headings(&mut page).is_empty(), "{:?}", labels(&mut page));

        toggle_group_by_type(&mut page);
        assert_eq!(headings(&mut page), Vec::<String>::new(), "{:?}", labels(&mut page));
        let flat = labels(&mut page);
        assert_eq!(
            flat[..2],
            ["chrome.exe (3)".to_string(), "notepad.exe".to_string()],
            "one CPU order across types, same-name groups kept: {flat:?}"
        );
        assert!(flat.contains(&"RuntimeBroker.exe".to_string()), "{flat:?}");
        assert_ne!(h.state::<ProcessesState>().sort_column, ProcessColumn::Name, "the toggle does not sort");

        let kept = || ProcessesSettings::new().unwrap().grouping().by_type().get();
        assert!(!kept(), "the choice is kept in the settings");

        toggle_group_by_type(&mut page);
        assert!(!headings(&mut page).is_empty(), "{:?}", labels(&mut page));
        assert!(kept());
    }

    #[guinea::test(iterations = 8, exclusive = "store")]
    fn a_selected_process_that_exits_stays_until_something_else_is_selected(h: &mut Harness) {
        let _store = start(h);
        let h = &*h;
        let mut page = mount(h);

        select(&mut page, "notepad.exe");
        assert_eq!(h.state::<ProcessesState>().selected, Some(NOTEPAD));
        assert!(end_task_enabled(&page));
        let alive = page.item_where(|item| label(item) == "notepad.exe").tree();
        assert_ne!(cell(&alive, ProcessColumn::Memory), "");

        report(h, without(&[NOTEPAD]));
        page.settle();

        let ghost = page
            .item_where(|item| label(item).starts_with("notepad.exe"))
            .tree();
        assert!(ghost.find(ProcessesMark::Exited).is_some(), "{ghost:#?}");
        assert_eq!(label(&ghost), "notepad.exe Exited");
        assert!(is_marked_selected(&ghost), "the exited process is still selected");
        assert_eq!(marked_selected(&mut page), ["notepad.exe Exited"]);
        let idle = page.item_where(|item| label(item) == "idle.exe").tree();
        assert_eq!(cell(&ghost, ProcessColumn::Cpu), cell(&idle, ProcessColumn::Cpu));
        assert_ne!(
            cell(&ghost, ProcessColumn::Memory),
            cell(&alive, ProcessColumn::Memory),
            "an exited process shows no memory"
        );
        assert!(!end_task_enabled(&page));
        assert!(
            page.find_text(&format!(
                "Selected: \u{2068}notepad.exe\u{2069} | PID \u{2068}{NOTEPAD}\u{2069} | Exited"
            ))
            .is_some(),
            "{:#?}",
            page.tree()
        );

        report(h, without(&[NOTEPAD]));
        page.settle();
        assert!(labels(&mut page).contains(&"notepad.exe Exited".to_string()));

        select(&mut page, "idle.exe");
        assert_eq!(h.state::<ProcessesState>().selected, Some(IDLE));
        assert!(
            !labels(&mut page).iter().any(|label| label.starts_with("notepad.exe")),
            "{:?}",
            labels(&mut page)
        );
    }

    #[guinea::test(iterations = 8, exclusive = "store")]
    fn a_process_that_exits_unselected_drops_out_at_once(h: &mut Harness) {
        let _store = start(h);
        let h = &*h;
        let mut page = mount(h);

        report(h, without(&[NOTEPAD]));
        page.settle();
        let after = labels(&mut page);
        assert!(!after.iter().any(|l| l.starts_with("notepad.exe")), "{after:?}");
        assert!(page.find(ProcessesMark::Exited).is_none(), "{after:?}");

        select(&mut page, "idle.exe");
        open(&mut page, "chrome.exe (3)");
        select(&mut page, "idle.exe");
        report(h, without(&[NOTEPAD, CHROME[1]]));
        page.settle();
        let after = labels(&mut page);
        assert!(after.contains(&"chrome.exe (2)".to_string()), "{after:?}");
        assert_eq!(
            after.iter().filter(|l| l.as_str() == "chrome.exe").count(),
            2,
            "{after:?}"
        );
        assert!(!after.iter().any(|l| l.contains("Exited")), "{after:?}");

        report(h, without(&[NOTEPAD, CHROME[1], IDLE]));
        page.settle();
        assert_eq!(marked_selected(&mut page), ["idle.exe Exited"]);
        assert_eq!(
            labels(&mut page).iter().filter(|l| l.contains("Exited")).count(),
            1,
            "only the selected one is held"
        );
    }

    #[guinea::test(iterations = 8, exclusive = "store")]
    fn a_selected_group_keeps_members_that_exit(h: &mut Harness) {
        let _store = start(h);
        let h = &*h;
        let mut page = mount(h);

        open(&mut page, "chrome.exe (3)");
        assert!(h.state::<ProcessesState>().selected.is_some_and(|pid| CHROME.contains(&pid)));
        let members = |page: &mut Mounted<'_, Processes>| {
            labels(page)
                .into_iter()
                .filter(|label| label.starts_with("chrome.exe") && !label.contains('('))
                .collect::<Vec<_>>()
        };
        assert_eq!(members(&mut page).len(), 3, "{:?}", labels(&mut page));

        report(h, without(&[CHROME[1]]));
        page.settle();
        let mut after = members(&mut page);
        after.sort();
        assert_eq!(
            after,
            ["chrome.exe", "chrome.exe", "chrome.exe Exited"],
            "{:?}",
            labels(&mut page)
        );
        assert!(labels(&mut page).contains(&"chrome.exe (3)".to_string()));
        let mut block = marked_selected(&mut page);
        block.sort();
        assert_eq!(
            block,
            ["chrome.exe", "chrome.exe", "chrome.exe (3)", "chrome.exe Exited"],
            "the whole group, the exited member with it, is still selected"
        );

        report(h, without(&CHROME));
        page.settle();
        let all = labels(&mut page);
        assert!(all.contains(&"chrome.exe Exited".to_string()), "{all:?}");
        assert_eq!(members(&mut page).len(), 4, "{all:?}");
        assert_eq!(
            marked_selected(&mut page),
            ["chrome.exe Exited"; 4],
            "an exited group is still selected as one block"
        );
        assert!(!end_task_enabled(&page));
    }

    #[guinea::test(iterations = 8, exclusive = "store")]
    fn a_selected_group_holds_its_place_when_the_rest_resorts(h: &mut Harness) {
        let _store = start(h);
        let h = &*h;
        let mut page = mount(h);

        let before = position(&mut page, "chrome.exe (3)").unwrap();
        select(&mut page, "chrome.exe (3)");

        let mut calmer = machine();
        for p in calmer.iter_mut() {
            p.cpu_percent = match p.pid {
                pid if CHROME.contains(&pid) => 0.0,
                IDLE => 9.0,
                CMD => 8.0,
                _ => p.cpu_percent,
            };
        }
        report(h, calmer.clone());
        page.settle();
        assert_eq!(position(&mut page, "chrome.exe (3)"), Some(before), "{:?}", labels(&mut page));

        select(&mut page, "idle.exe");
        report(h, calmer);
        page.settle();
        assert_ne!(position(&mut page, "chrome.exe (3)"), Some(before), "{:?}", labels(&mut page));
    }

    #[guinea::test(iterations = 8, exclusive = "store")]
    fn a_group_is_selected_as_one_block_and_a_member_alone(h: &mut Harness) {
        let _store = start(h);
        let h = &*h;
        let mut page = mount(h);

        open(&mut page, "chrome.exe (3)");
        assert_eq!(
            marked_selected(&mut page),
            ["chrome.exe (3)", "chrome.exe", "chrome.exe", "chrome.exe"]
        );
        assert_eq!(
            painted(&mut page),
            ["chrome.exe (3)", "chrome.exe", "chrome.exe", "chrome.exe"],
            "the plate runs under the whole group"
        );

        let member = page
            .items()
            .iter()
            .position(|item| label(item) == "chrome.exe")
            .unwrap();
        page.item(member + 1).click_here();
        page.settle();
        assert_eq!(marked_selected(&mut page), ["chrome.exe"]);
        assert_eq!(painted(&mut page), ["chrome.exe"]);
        assert!(h.state::<ProcessesState>().selected.is_some_and(|pid| CHROME.contains(&pid)));
    }

    #[guinea::test(iterations = 8, exclusive = "store")]
    fn a_chevron_opens_windows_services_and_consoles(h: &mut Harness) {
        let _store = start(h);
        let h = &*h;
        let mut page = mount(h);

        let before = labels(&mut page);
        assert!(!before.iter().any(|l| l == "Untitled - Notepad"), "{before:?}");
        assert!(!before.iter().any(|l| l == "conhost.exe"), "{before:?}");
        assert!(!before.iter().any(|l| l == "Windows Audio"), "{before:?}");

        for row in ["chrome.exe (3)", "notepad.exe", "DHCP Client +1 — svchost.exe", "cmd.exe"] {
            assert!(
                chevron_takes_the_whole_slot(&mut page, row),
                "{row}: a chevron with no background takes the pointer only on its glyph"
            );
        }

        open(&mut page, "notepad.exe");
        open(&mut page, "DHCP Client +1 — svchost.exe");
        open(&mut page, "cmd.exe");

        let after = labels(&mut page);
        let under = |parent: &str, child: &str| {
            let at = after.iter().position(|l| l == parent).unwrap();
            after[at + 1..].iter().take(2).any(|l| l == child)
        };
        assert!(under("notepad.exe", "Untitled - Notepad"), "{after:?}");
        assert!(under("DHCP Client +1 — svchost.exe", "DHCP Client"), "{after:?}");
        assert!(under("DHCP Client +1 — svchost.exe", "Windows Audio"), "{after:?}");
        assert!(under("cmd.exe", "conhost.exe"), "{after:?}");
    }

    #[guinea::test(iterations = 8, exclusive = "store")]
    fn a_header_sorts_and_a_heading_folds_its_section(h: &mut Harness) {
        let _store = start(h);
        let h = &*h;
        let mut page = mount(h);

        let by_cpu = labels(&mut page);
        page.click(ProcessColumn::Memory);
        page.settle();
        let state = h.state::<ProcessesState>();
        assert_eq!((state.sort_column, state.descending), (ProcessColumn::Memory, true));
        let by_memory = labels(&mut page);
        assert_ne!(by_cpu, by_memory);
        let idle = by_memory.iter().position(|l| l == "idle.exe").unwrap();
        let cmd = by_memory.iter().position(|l| l == "cmd.exe").unwrap();
        assert!(idle < cmd, "{by_memory:?}");

        page.click(ProcessColumn::Memory);
        page.settle();
        let state = h.state::<ProcessesState>();
        assert_eq!((state.sort_column, state.descending), (ProcessColumn::Memory, false));
        let ascending = labels(&mut page);
        let idle = ascending.iter().position(|l| l == "idle.exe").unwrap();
        let cmd = ascending.iter().position(|l| l == "cmd.exe").unwrap();
        assert!(cmd < idle, "{ascending:?}");

        let heading = by_memory
            .iter()
            .find(|l| l.starts_with("Background processes (Microsoft)"))
            .unwrap()
            .clone();
        select(&mut page, &heading);
        let folded = labels(&mut page);
        select(&mut page, &heading);
        let unfolded = labels(&mut page);
        assert!(folded.contains(&heading), "{folded:?}");
        assert!(!folded.iter().any(|l| l == "RuntimeBroker.exe"), "{folded:?}");
        assert!(unfolded.iter().any(|l| l == "RuntimeBroker.exe"), "{unfolded:?}");
        assert_eq!(h.state::<ProcessesState>().selected, None, "a heading is not selectable");
    }

    #[guinea::test(iterations = 8, exclusive = "store")]
    fn end_task_keeps_the_selection(h: &mut Harness) {
        let _store = start(h);
        let h = &*h;
        let mut page = mount(h);

        select(&mut page, "notepad.exe");
        let end_task = page.click(ProcessesMark::EndTask);
        end_task.settle();
        assert!(end_task.chain().published::<WindowsActionRequest>(), "{:#?}", end_task.chain());
        page.settle();
        assert_eq!(h.state::<ProcessesState>().selected, Some(NOTEPAD));
        assert!(end_task_enabled(&page));

        report(h, without(&[NOTEPAD]));
        page.settle();
        assert_eq!(h.state::<ProcessesState>().selected, Some(NOTEPAD));
        assert_eq!(marked_selected(&mut page), ["notepad.exe Exited"]);
        assert!(!end_task_enabled(&page));
    }

    #[guinea::test(iterations = 8, exclusive = "store")]
    fn an_app_whose_window_closes_before_it_exits(h: &mut Harness) {
        let _store = start(h);
        let h = &*h;
        let mut page = mount(h);

        let section_of = |labels: &[String], wanted: &str| {
            let at = labels.iter().position(|l| l == wanted).unwrap();
            labels[..at]
                .iter()
                .rev()
                .find(|l| l.starts_with("Apps") || l.starts_with("Background") || l.starts_with("Services"))
                .cloned()
                .unwrap()
        };

        select(&mut page, "notepad.exe");
        NOTEPAD_WINDOW.set(false);
        report(h, machine());
        page.settle();
        let windowless = labels(&mut page);
        assert!(section_of(&windowless, "notepad.exe").starts_with("Apps"), "{windowless:?}");

        report(h, without(&[NOTEPAD]));
        page.settle();
        let ghost = labels(&mut page);
        assert!(section_of(&ghost, "notepad.exe Exited").starts_with("Apps"), "{ghost:?}");
        assert_eq!(marked_selected(&mut page), ["notepad.exe Exited"]);

        report(h, machine());
        page.settle();
        select(&mut page, "idle.exe");
        let released = labels(&mut page);
        assert!(
            section_of(&released, "notepad.exe").starts_with("Background processes ("),
            "{released:?}"
        );
    }

    fn overlay(page: &Mounted<'_, Processes>) -> Option<&'static str> {
        ["Connecting...", "Can't reach the agent. Still trying."]
            .into_iter()
            .find(|text| page.find_text(text).is_some())
    }

    #[guinea::test(iterations = 16, exclusive = "store")]
    fn a_lost_agent_is_retried_behind_an_overlay_not_the_splash(h: &mut Harness) {
        let _store = start(h);
        crate::test_agent::reset(true);
        crate::test_agent::serve(WindowsReport {
            processes: machine(),
            ..Default::default()
        });
        h.feature(crate::test_agent::FakeAgentFeature).unwrap();
        let h = &*h;
        h.install::<AgentLinkFeature>(&AgentLinkDeps {
            start_in_process: crate::test_agent::start_in_process,
        })
            .unwrap();
        let params = crate::routes::ProcessesParams::default();
        let mut page = Mounted::mount_with(&h.segment(), params, |page| {
            View::provide(scheme_context(), ColorScheme::Dark, page)
        })
        .unwrap();
        let after = |seconds: u64, page: &mut Mounted<'_, Processes>| {
            h.advance(Duration::from_secs(seconds));
            page.settle();
        };

        after(1, &mut page);
        assert_eq!(overlay(&page), None, "{:#?}", page.tree());
        assert!(labels(&mut page).contains(&"notepad.exe".to_string()));

        crate::test_agent::set_up(false);
        after(2, &mut page);
        assert_eq!(overlay(&page), Some("Connecting..."), "{:#?}", page.tree());
        assert!(labels(&mut page).contains(&"notepad.exe".to_string()), "the last rows stay");

        after(25, &mut page);
        assert_eq!(
            overlay(&page),
            Some("Can't reach the agent. Still trying."),
            "connects: {}",
            crate::test_agent::connects()
        );
        assert!(!h.state::<AgentLinkState>().awaiting_first_connection(), "no splash again");

        crate::test_agent::set_up(true);
        after(6, &mut page);
        assert_eq!(overlay(&page), None, "{:#?}", page.tree());
    }

    #[guinea::test(iterations = 4, exclusive = "store")]
    fn a_chevron_click_also_selects_its_row(h: &mut Harness) {
        let _store = start(h);
        let h = &*h;
        let mut page = mount(h);

        open(&mut page, "notepad.exe");
        assert_eq!(h.state::<ProcessesState>().selected, Some(NOTEPAD));
    }

    struct EmptyArea;

    impl guinea::Mark for EmptyArea {
        fn name(&self) -> &'static str {
            guinea_widgets::table::EMPTY_AREA
        }
    }

    #[guinea::test(iterations = 4, exclusive = "store")]
    fn a_click_below_the_rows_drops_the_selection(h: &mut Harness) {
        let _store = start(h);
        let h = &*h;
        let mut page = mount(h);

        select(&mut page, "chrome.exe (3)");
        assert_eq!(h.state::<ProcessesState>().selected, Some(CHROME[0]));

        page.click(EmptyArea).settle();
        page.settle();

        assert_eq!(h.state::<ProcessesState>().selected, None);
        assert!(marked_selected(&mut page).is_empty());
        assert!(!end_task_enabled(&page));
    }

    #[guinea::test(iterations = 4, exclusive = "store")]
    fn a_click_on_the_page_around_the_table_drops_the_selection(h: &mut Harness) {
        let _store = start(h);
        let h = &*h;
        let mut page = mount(h);

        for blank in [PageMark::Header, PageMark::Status, PageMark::Blank] {
            select(&mut page, "notepad.exe");
            assert_eq!(h.state::<ProcessesState>().selected, Some(NOTEPAD));

            page.click(blank).settle();
            page.settle();

            assert_eq!(h.state::<ProcessesState>().selected, None, "{blank:?}");
        }
    }

    #[guinea::test(iterations = 4, exclusive = "store")]
    fn a_press_away_from_the_table_drops_the_selection(h: &mut Harness) {
        let _store = start(h);
        let h = &*h;
        let mut page = mount(h);
        select(&mut page, "notepad.exe");
        assert!(!marked_selected(&mut page).is_empty());

        h.publish(PressedAway).settle();
        page.settle();

        assert_eq!(h.state::<ProcessesState>().selected, None);
        assert_eq!(marked_selected(&mut page), Vec::<String>::new());
    }

    fn keepers<'a>(node: &'a Node, out: &mut Vec<&'a Node>) {
        if node.id.as_deref() == Some(guinea::Mark::name(&SelectionMark::Keeper)) {
            out.push(node);
        }
        for child in &node.children {
            keepers(child, out);
        }
    }

    #[guinea::test(iterations = 4, exclusive = "store")]
    fn the_table_its_menu_and_end_task_keep_the_selection(h: &mut Harness) {
        let _store = start(h);
        let h = &*h;
        let mut page = mount(h);
        right_click(&mut page, "notepad.exe");
        assert!(menu_open(&page), "{:#?}", page.tree());

        let tree = page.tree();
        let mut found = Vec::new();
        keepers(&tree, &mut found);
        let is_end_task =
            |node: &Node| node.id.as_deref() == Some(guinea::Mark::name(&ProcessesMark::EndTask));

        assert!(found.iter().any(|keeper| contains(keeper, &is_list)), "{tree:#?}");
        assert!(found.iter().any(|keeper| contains(keeper, &is_backdrop)), "{tree:#?}");
        assert!(found.iter().any(|keeper| contains(keeper, &is_end_task)), "{tree:#?}");
        assert!(!found.iter().any(|keeper| contains(keeper, &|node: &Node| {
            node.id.as_deref() == Some(guinea::Mark::name(&PageMark::Status))
        })), "{tree:#?}");
    }

    #[guinea::test(iterations = 4, exclusive = "store")]
    fn the_selection_bar_runs_through_the_window_rows_of_its_block(h: &mut Harness) {
        let _store = start(h);
        let h = &*h;
        let mut page = mount(h);

        open(&mut page, "notepad.exe");

        assert_eq!(marked_selected(&mut page), ["notepad.exe", "Untitled - Notepad"]);
    }

    fn right_click(page: &mut Mounted<'_, Processes>, wanted: &str) {
        page.send(ProcessesMsg::MenuAnchor { x: 40.0, y: 60.0 });
        select(page, wanted);
    }

    fn menu_open(page: &Mounted<'_, Processes>) -> bool {
        page.find(ProcessesMark::Menu).is_some()
    }

    #[guinea::test(iterations = 4, exclusive = "store")]
    fn a_right_click_opens_the_menu_of_that_process_and_suspend_goes_to_the_agent(h: &mut Harness) {
        let _store = start(h);
        let h = &*h;
        let mut page = mount(h);

        right_click(&mut page, "notepad.exe");
        assert!(menu_open(&page), "{:#?}", page.tree());
        assert_eq!(h.state::<ProcessesState>().selected, Some(NOTEPAD));

        let suspend = page.click(ProcessesMark::MenuSuspend);
        suspend.settle();
        assert!(suspend.chain().published::<WindowsActionRequest>(), "{:#?}", suspend.chain());
        page.settle();
        assert!(!menu_open(&page), "a command closes the menu");
    }

    #[guinea::test(iterations = 4, exclusive = "store")]
    fn a_left_click_opens_no_menu(h: &mut Harness) {
        let _store = start(h);
        let h = &*h;
        let mut page = mount(h);

        select(&mut page, "notepad.exe");
        assert!(!menu_open(&page));

        right_click(&mut page, "notepad.exe");
        page.send(ProcessesMsg::MenuDismiss);
        page.settle();
        assert!(!menu_open(&page));
    }

    fn contains(node: &Node, test: &dyn Fn(&Node) -> bool) -> bool {
        test(node) || node.children.iter().any(|child| contains(child, test))
    }

    fn is_list(node: &Node) -> bool {
        node.kind == "ScrollViewer"
    }

    fn is_backdrop(node: &Node) -> bool {
        node.id.as_deref() == Some(guinea::Mark::name(&ProcessesMark::MenuBackdrop))
    }

    fn layer_over_list(node: &Node) -> Option<&Node> {
        let list = node.children.iter().position(|child| contains(child, &is_list));
        let layer = node.children.iter().position(|child| contains(child, &is_backdrop));
        match (list, layer) {
            (Some(list), Some(layer)) if list != layer => (layer > list).then(|| &node.children[layer]),
            _ => node.children.iter().find_map(layer_over_list),
        }
    }

    fn path_to<'a>(node: &'a Node, test: &dyn Fn(&Node) -> bool, out: &mut Vec<&'a Node>) -> bool {
        out.push(node);
        if test(node) || node.children.iter().any(|child| path_to(child, test, out)) {
            return true;
        }
        out.pop();
        false
    }

    #[guinea::test(iterations = 4, exclusive = "store")]
    fn an_open_menu_lies_over_the_whole_list_so_the_wheel_cannot_reach_it(h: &mut Harness) {
        let _store = start(h);
        let h = &*h;
        let mut page = mount(h);

        right_click(&mut page, "notepad.exe");
        let tree = page.tree();
        let layer = layer_over_list(&tree).expect("the menu's layer comes after the list, over it");

        let mut path = Vec::new();
        assert!(path_to(layer, &is_backdrop, &mut path));
        for node in &path {
            for sized in [
                PropertyId::Width,
                PropertyId::Height,
                PropertyId::HorizontalAlignment,
                PropertyId::VerticalAlignment,
            ] {
                assert_eq!(page.property(node.at, sized), None, "{} {sized:?}", node.kind);
            }
        }
        let backdrop = path.last().unwrap();
        assert!(
            page.property(backdrop.at, PropertyId::BorderBackground).is_some(),
            "a layer without a background lets the pointer and the wheel through"
        );
    }

    #[guinea::test(iterations = 4, exclusive = "store")]
    fn an_open_menu_covers_the_table_and_a_click_on_it_only_closes_the_menu(h: &mut Harness) {
        let _store = start(h);
        let h = &*h;
        let mut page = mount(h);

        right_click(&mut page, "notepad.exe");
        assert!(page.find(ProcessesMark::MenuBackdrop).is_some());

        page.click(ProcessesMark::MenuBackdrop).settle();
        page.settle();

        assert!(!menu_open(&page));
        assert!(page.find(ProcessesMark::MenuBackdrop).is_none());
        assert_eq!(h.state::<ProcessesState>().selected, Some(NOTEPAD));
    }

    #[guinea::test(iterations = 4, exclusive = "store")]
    fn menu_commands_run_on_the_process_the_menu_was_opened_for(h: &mut Harness) {
        let _store = start(h);
        let h = &*h;
        let mut page = mount(h);

        right_click(&mut page, "notepad.exe");
        page.click(ProcessesMark::MenuSearchOnline).settle();
        right_click(&mut page, "chrome.exe (3)");
        page.click(ProcessesMark::MenuSearchOnline).settle();

        assert_eq!(
            shell_requests(),
            [
                ShellRequest::SearchOnline("notepad.exe".into()),
                ShellRequest::SearchOnline("chrome.exe".into()),
            ]
        );
    }

    #[guinea::test(iterations = 4, exclusive = "store")]
    fn a_group_menu_offers_nothing_that_would_hit_only_one_member(h: &mut Harness) {
        let _store = start(h);
        let h = &*h;
        let mut page = mount(h);

        right_click(&mut page, "chrome.exe (3)");
        assert!(menu_open(&page), "{:#?}", page.tree());
        for per_process in [
            ProcessesMark::MenuEndTask,
            ProcessesMark::MenuSuspend,
            ProcessesMark::MenuResume,
        ] {
            assert!(page.find(per_process).is_none(), "{per_process:?}");
        }
        assert!(page.find(ProcessesMark::MenuSearchOnline).is_some());
    }

    #[guinea::test(iterations = 4, exclusive = "store")]
    fn a_window_row_menu_acts_on_the_window(h: &mut Harness) {
        let _store = start(h);
        let h = &*h;
        let mut page = mount(h);

        open(&mut page, "notepad.exe");
        right_click(&mut page, "Untitled - Notepad");
        assert!(page.find(ProcessesMark::MenuEndTask).is_none());
        page.click(ProcessesMark::MenuSwitchTo).settle();

        assert_eq!(
            shell_requests(),
            [ShellRequest::Window {
                handle: 0,
                command: WindowCommand::SwitchTo,
            }]
        );
    }

    #[guinea::test(iterations = 4, exclusive = "store")]
    fn dropping_the_selection_closes_the_menu(h: &mut Harness) {
        let _store = start(h);
        let h = &*h;
        let mut page = mount(h);

        right_click(&mut page, "notepad.exe");
        assert!(menu_open(&page), "{:#?}", page.tree());
        h.publish(PressedAway).settle();
        page.settle();
        assert!(!menu_open(&page), "{:#?}", page.tree());

        right_click(&mut page, "notepad.exe");
        assert!(menu_open(&page), "{:#?}", page.tree());
        h.dispatch::<ProcessesState>().emit(Deselect);
        page.settle();
        assert!(!menu_open(&page), "{:#?}", page.tree());

        select(&mut page, "notepad.exe");
        assert!(!menu_open(&page), "a later selection does not bring the old menu back");
    }

    const KERNEL: u32 = 4;

    fn with_kernel() -> Vec<WindowsProcessStats> {
        let mut system = process(KERNEL, "System", 0.4);
        system.is_kernel_process = true;
        system.signature = SignatureStatus::Microsoft;
        let mut rows = machine();
        rows.push(system);
        rows
    }

    fn disabled(page: &Mounted<'_, Processes>, mark: ProcessesMark) -> bool {
        let node = page.find(mark).unwrap_or_else(|| panic!("{mark:?} is on the page"));
        page.property(node, PropertyId::ButtonIsEnabled) == Some(&PropertyValue::Bool(false))
    }

    #[guinea::test(iterations = 4, exclusive = "store")]
    fn a_kernel_process_offers_nothing_to_do_to_it(h: &mut Harness) {
        let _store = start(h);
        let h = &*h;
        let mut page = mount(h);
        report(h, with_kernel());
        page.settle();
        let asked = Rc::new(Cell::new(0));
        let counted = asked.clone();
        let _watch = GlobalEventBus::subscribe_fn(move |_: WindowsActionRequest| counted.set(counted.get() + 1));

        right_click(&mut page, "System");
        assert_eq!(h.state::<ProcessesState>().selected, Some(KERNEL));
        assert!(!end_task_enabled(&page));
        for mark in [ProcessesMark::MenuEndTask, ProcessesMark::MenuSuspend, ProcessesMark::MenuResume] {
            assert!(disabled(&page, mark), "{mark:?}");
        }
        assert!(!disabled(&page, ProcessesMark::MenuPin));

        let dispatch = h.dispatch::<ProcessesState>();
        dispatch.emit(Terminate);
        dispatch.emit(RunProcessCommand(ProcessCommand::Suspend));
        page.settle();
        assert_eq!(asked.get(), 0, "the actor refuses what the buttons do not offer");

        select(&mut page, "notepad.exe");
        assert!(end_task_enabled(&page));
    }

    #[guinea::test(iterations = 4, exclusive = "store")]
    fn a_pin_that_is_not_running_stays_until_unpinned(h: &mut Harness) {
        let _store = start(h);
        let h = &*h;
        let mut page = mount(h);
        pin_from_menu(&mut page, "notepad.exe", ProcessesMark::MenuPin);
        select(&mut page, "chrome.exe (3)");

        report(h, without(&[NOTEPAD]));
        page.settle();

        let all = labels(&mut page);
        assert_eq!(all[..2], ["Pinned (1)".to_string(), "notepad.exe Not running".to_string()], "{all:?}");
        let selected = h.state::<ProcessesState>().selected;
        page.item_where(|item| label(item) == "notepad.exe Not running").click_here();
        page.settle();
        assert_eq!(h.state::<ProcessesState>().selected, selected, "a placeholder is not selected");

        right_click(&mut page, "notepad.exe Not running");
        assert!(menu_open(&page), "{:#?}", page.tree());
        assert!(page.find(ProcessesMark::MenuEndTask).is_none());
        for mark in [ProcessesMark::MenuOpenFileLocation, ProcessesMark::MenuProperties, ProcessesMark::MenuSearchOnline] {
            assert!(!disabled(&page, mark), "{mark:?}: the image is still on disk");
        }
        page.click(ProcessesMark::MenuOpenFileLocation).settle();
        page.settle();
        right_click(&mut page, "notepad.exe Not running");
        page.click(ProcessesMark::MenuProperties).settle();
        page.settle();
        assert_eq!(
            shell_requests(),
            [
                ShellRequest::RevealFile(NOTEPAD_PATH.into()),
                ShellRequest::FileProperties(NOTEPAD_PATH.into()),
            ],
            "a pin that is not running is still an image with a path"
        );

        right_click(&mut page, "notepad.exe Not running");
        page.click(ProcessesMark::MenuUnpin).settle();
        page.settle();

        let all = labels(&mut page);
        assert!(!all.iter().any(|label| label.starts_with("Pinned") || label.starts_with("notepad")), "{all:?}");
    }

    fn column_menu(page: &mut Mounted<'_, Processes>) {
        page.send(ProcessesMsg::ColumnMenu);
        page.send(ProcessesMsg::MenuAnchor { x: 40.0, y: 10.0 });
        page.settle();
    }

    fn pid_shown() -> Option<bool> {
        ProcessesSettings::new().unwrap().columns().configs().get("pid").map(|config| config.visible)
    }

    #[guinea::test(iterations = 4, exclusive = "store")]
    fn pid_and_process_name_come_from_the_header_menu(h: &mut Harness) {
        let _store = start(h);
        let h = &*h;
        let mut page = mount(h);
        let notepad = |page: &mut Mounted<'_, Processes>| page.item_where(|item| label(item) == "notepad.exe").tree();
        assert!(notepad(&mut page).find(ProcessColumn::Pid).is_none(), "PID is hidden until asked for");
        assert!(notepad(&mut page).find(ProcessColumn::ProcessName).is_none());

        column_menu(&mut page);
        assert!(menu_open(&page), "{:#?}", page.tree());
        assert!(page.find(ProcessesMark::MenuPin).is_none(), "the header menu is not a process menu");
        page.click(ProcessesMark::MenuColumnPid).settle();
        page.settle();
        column_menu(&mut page);
        page.click(ProcessesMark::MenuColumnProcessName).settle();
        page.settle();

        assert!(!menu_open(&page));
        let row = notepad(&mut page);
        assert_eq!(cell(&row, ProcessColumn::Pid), NOTEPAD.to_string());
        assert_eq!(cell(&row, ProcessColumn::ProcessName), "notepad.exe");
        let group = page.item_where(|item| label(item) == "chrome.exe (3)").tree();
        assert_eq!(cell(&group, ProcessColumn::Pid), "", "a group of three has no single PID");
        assert_eq!(cell(&group, ProcessColumn::ProcessName), "chrome.exe");
        assert_eq!(pid_shown(), Some(true), "the choice is kept in the settings");

        column_menu(&mut page);
        page.click(ProcessesMark::MenuColumnPid).settle();
        page.settle();
        assert!(notepad(&mut page).find(ProcessColumn::Pid).is_none());
        assert_eq!(pid_shown(), Some(false));
    }

    #[guinea::test(iterations = 4, exclusive = "store")]
    fn a_press_away_closes_the_header_menu(h: &mut Harness) {
        let _store = start(h);
        let h = &*h;
        let mut page = mount(h);
        assert_eq!(h.state::<ProcessesState>().selected, None);

        column_menu(&mut page);
        assert!(menu_open(&page), "{:#?}", page.tree());
        h.publish(PressedAway).settle();
        page.settle();
        assert!(!menu_open(&page), "{:#?}", page.tree());

        select(&mut page, "notepad.exe");
        column_menu(&mut page);
        assert!(menu_open(&page), "{:#?}", page.tree());
        h.publish(PressedAway).settle();
        page.settle();
        assert!(!menu_open(&page), "with a selection too");
    }

    #[guinea::test(iterations = 4, exclusive = "store")]
    fn sorting_by_pid_starts_with_the_lowest(h: &mut Harness) {
        let _store = start(h);
        let h = &*h;
        let mut page = mount(h);
        column_menu(&mut page);
        page.click(ProcessesMark::MenuColumnPid).settle();
        page.settle();
        toggle_group_by_type(&mut page);

        h.dispatch::<ProcessesState>().emit(Sort(ProcessColumn::Pid));
        page.settle();
        assert!(!h.state::<ProcessesState>().descending);

        let pids: Vec<u32> = page
            .items()
            .iter()
            .filter_map(|item| cell(item, ProcessColumn::Pid).parse().ok())
            .collect();
        assert!(pids.len() > 2, "{pids:?}");
        assert!(pids.is_sorted(), "{pids:?}");
    }

    fn pin_from_menu(page: &mut Mounted<'_, Processes>, wanted: &str, mark: ProcessesMark) {
        right_click(page, wanted);
        page.click(mark).settle();
        page.settle();
    }

    fn pin_kept(name: &str) -> Option<PinnedProcess> {
        ProcessesSettings::new().unwrap().grouping().pins().get(&name.to_string())
    }

    #[guinea::test(iterations = 4, exclusive = "store")]
    fn pinning_from_the_menu_moves_the_process_into_a_pinned_section_on_top(h: &mut Harness) {
        let _store = start(h);
        let h = &*h;
        let mut page = mount(h);

        pin_from_menu(&mut page, "notepad.exe", ProcessesMark::MenuPin);
        let all = labels(&mut page);
        assert_eq!(all[..2], ["Pinned (1)".to_string(), "notepad.exe".to_string()], "{all:?}");
        assert_eq!(all.iter().filter(|label| *label == "notepad.exe").count(), 1, "{all:?}");
        assert_eq!(
            pin_kept("notepad.exe").map(|pin| pin.exe_path),
            Some(NOTEPAD_PATH.to_string()),
            "the pin is kept in the settings with the path its icon comes from"
        );

        right_click(&mut page, "notepad.exe");
        assert!(page.find(ProcessesMark::MenuPin).is_none(), "a pinned process offers Unpin");
        page.click(ProcessesMark::MenuUnpin).settle();
        page.settle();

        let all = labels(&mut page);
        assert!(!all.iter().any(|label| label.starts_with("Pinned")), "{all:?}");
        assert_eq!(pin_kept("notepad.exe"), None);
    }

    #[guinea::test(iterations = 4, exclusive = "store")]
    fn pinning_a_group_pins_every_member(h: &mut Harness) {
        let _store = start(h);
        let h = &*h;
        let mut page = mount(h);

        pin_from_menu(&mut page, "chrome.exe (3)", ProcessesMark::MenuPin);

        let all = labels(&mut page);
        assert_eq!(all[..2], ["Pinned (1)".to_string(), "chrome.exe (3)".to_string()], "{all:?}");
    }

    #[guinea::test(iterations = 4, exclusive = "store")]
    fn without_grouping_by_type_pinned_processes_sit_on_top_above_a_rule(h: &mut Harness) {
        let _store = start(h);
        let h = &*h;
        let mut page = mount(h);
        toggle_group_by_type(&mut page);

        pin_from_menu(&mut page, "RuntimeBroker.exe", ProcessesMark::MenuPin);

        let all = labels(&mut page);
        assert_eq!(all[0], "RuntimeBroker.exe", "{all:?}");
        assert_eq!(headings(&mut page), Vec::<String>::new());
        let ruled: Vec<String> = page
            .items()
            .iter()
            .filter(|item| item.find(ProcessesMark::PinnedRule).is_some())
            .map(label)
            .collect();
        assert_eq!(ruled, ["RuntimeBroker.exe"]);
    }

    #[guinea::test(iterations = 4, exclusive = "store")]
    fn a_window_row_cannot_be_pinned(h: &mut Harness) {
        let _store = start(h);
        let h = &*h;
        let mut page = mount(h);

        open(&mut page, "notepad.exe");
        right_click(&mut page, "Untitled - Notepad");

        assert!(page.find(ProcessesMark::MenuPin).is_none());
        assert!(page.find(ProcessesMark::MenuUnpin).is_none());
    }

    #[guinea::test(iterations = 4, exclusive = "store")]
    fn a_right_click_on_a_section_heading_does_not_collapse_it(h: &mut Harness) {
        let _store = start(h);
        let h = &*h;
        let mut page = mount(h);

        let heading = labels(&mut page)
            .into_iter()
            .find(|label| label.starts_with("Apps"))
            .unwrap();
        right_click(&mut page, &heading);

        assert!(!menu_open(&page));
        assert!(labels(&mut page).contains(&"notepad.exe".to_string()));
    }

    const VM: u32 = 70;

    fn with_vm() -> Vec<WindowsProcessStats> {
        let mut rows = machine();
        rows.push(process(VM, "vmmemWSL", 1.5));
        rows
    }

    fn linux(global_pid: u32, local_pid: u32, mnt_ns: u64, name: &str) -> LinuxProcessStats {
        LinuxProcessStats {
            global_pid,
            local_pid,
            mnt_ns,
            pid_ns: 1,
            name: name.into(),
            ..Default::default()
        }
    }

    fn mounts(mnt_ns: u64, kind: EnvironmentKind, name: &str) -> LinuxEnvironmentInfo {
        LinuxEnvironmentInfo {
            mnt_ns,
            pid_ns: 1,
            kind,
            name: name.into(),
        }
    }

    fn linux_report(h: &Harness) {
        h.publish(RemoteScanResult::Scan(RemoteScan {
            schema_id: "wsl",
            processes: vec![linux(NOTEPAD, 1, 1, "init"), linux(90, 2, 1, "bash"), linux(95, 3, 2, "cupsd")],
            machine: Default::default(),
            environments: vec![
                mounts(2, EnvironmentKind::Unknown, ""),
                mounts(1, EnvironmentKind::CurrentDistro, "Ubuntu"),
            ],
            docker_containers: Vec::new(),
        }))
        .settle();
    }

    fn wsl_page(h: &Harness) -> Mounted<'_, Processes> {
        let mut page = mount(h);
        report(h, with_vm());
        linux_report(h);
        page.settle();
        page
    }

    #[guinea::test(iterations = 4, exclusive = "store")]
    fn wsl_is_a_section_headed_by_the_vm_whose_distributions_open_to_their_processes(h: &mut Harness) {
        let _store = start(h);
        let h = &*h;
        let mut page = wsl_page(h);

        let all = labels(&mut page);
        let at = all.iter().position(|label| label == "WSL (1)").unwrap_or_else(|| panic!("{all:?}"));
        assert_eq!(all[at + 1], "Ubuntu (3)", "a service with private mounts is Ubuntu's: {all:?}");
        assert!(all[at + 2].starts_with("Background processes (Microsoft)"), "nothing else in the section: {all:?}");
        assert!(!all.iter().any(|label| label.starts_with("vmmemWSL")), "the VM is the heading: {all:?}");

        select(&mut page, "Ubuntu (3)");
        let all = labels(&mut page);
        let at = position(&mut page, "Ubuntu (3)").unwrap();
        assert_eq!(
            all[at + 1..at + 4],
            ["bash".to_string(), "cupsd".to_string(), "init".to_string()],
            "{all:?}"
        );
        assert_eq!(h.state::<ProcessesState>().selected, None, "a distribution opens, it is not selected");
        assert_eq!(h.state::<ProcessesState>().selected_linux, None);

        select(&mut page, "Ubuntu (3)");
        assert!(!labels(&mut page).contains(&"init".to_string()));
    }

    #[guinea::test(iterations = 4, exclusive = "store")]
    fn without_grouping_by_type_the_vm_opens_to_its_distributions(h: &mut Harness) {
        let _store = start(h);
        let h = &*h;
        let mut page = wsl_page(h);
        toggle_group_by_type(&mut page);

        let closed = labels(&mut page);
        assert!(closed.contains(&"vmmemWSL".to_string()), "{closed:?}");
        assert!(!closed.contains(&"Ubuntu (3)".to_string()), "{closed:?}");

        open(&mut page, "vmmemWSL");
        let all = labels(&mut page);
        let at = position(&mut page, "vmmemWSL").unwrap();
        assert_eq!(all[at + 1], "Ubuntu (3)", "{all:?}");
        assert_eq!(
            name_inset(&mut page, "Ubuntu (3)"),
            name_inset(&mut page, "notepad.exe"),
            "a distribution sits level with the VM and every other process"
        );
        assert_eq!(marked_selected(&mut page), ["vmmemWSL"], "the VM is selected alone");

        select(&mut page, "Ubuntu (3)");
        let all = labels(&mut page);
        assert_eq!(all[at + 2..at + 5], ["bash".to_string(), "cupsd".to_string(), "init".to_string()], "{all:?}");
        assert_eq!(
            name_inset(&mut page, "init"),
            name_inset(&mut page, "Ubuntu (3)"),
            "Linux processes stay flush with their distribution"
        );

        open(&mut page, "vmmemWSL");
        assert!(!labels(&mut page).contains(&"Ubuntu (3)".to_string()));
    }

    fn is_reserve(node: &Node) -> bool {
        node.id.as_deref() == Some(guinea::Mark::name(&ProcessesMark::StatusReserve))
    }

    fn shown_texts(node: &Node, out: &mut Vec<String>) {
        if is_reserve(node) {
            return;
        }
        if let Some(text) = &node.text {
            out.push(text.clone());
        }
        for child in &node.children {
            shown_texts(child, out);
        }
    }

    fn status(page: &Mounted<'_, Processes>) -> String {
        let tree = page.tree();
        let bar = tree.find(PageMark::Status).expect("the page has a status bar");
        let mut said = Vec::new();
        shown_texts(bar, &mut said);
        said.join(" ").replace(['\u{2068}', '\u{2069}'], "")
    }

    #[guinea::test(iterations = 4, exclusive = "store")]
    fn the_status_bar_counts_what_the_table_holds(h: &mut Harness) {
        let _store = start(h);
        let h = &*h;
        let mut page = mount(h);
        assert_eq!(status(&page), "1 app 7 background 1 service 0 kernel 9 processes");
        assert_eq!(shares(&page), 3, "no segment for an empty part");

        report(h, with_vm());
        linux_report(h);
        page.settle();

        assert_eq!(status(&page), "1 app 8 background 1 service 0 kernel 3 WSL 13 processes");
        assert_eq!(shares(&page), 4);
    }

    fn shares(page: &Mounted<'_, Processes>) -> usize {
        let tree = page.tree();
        let mut found = Vec::new();
        marked(tree.find(PageMark::Status).expect("a status bar"), ProcessesMark::StatusShare, &mut found);
        found.len()
    }

    fn marked<'a>(node: &'a Node, mark: ProcessesMark, out: &mut Vec<&'a Node>) {
        if node.id.as_deref() == Some(guinea::Mark::name(&mark)) {
            out.push(node);
        }
        for child in &node.children {
            marked(child, mark, out);
        }
    }

    #[guinea::test(iterations = 4, exclusive = "store")]
    fn every_status_part_reserves_room_for_its_widest_count(h: &mut Harness) {
        let _store = start(h);
        let h = &*h;
        let page = mount(h);
        let tree = page.tree();
        let bar = tree.find(PageMark::Status).expect("the page has a status bar");
        let mut reserves = Vec::new();
        marked(bar, ProcessesMark::StatusReserve, &mut reserves);

        let widest: Vec<String> = reserves
            .iter()
            .map(|reserve| reserve.text.clone().unwrap_or_default().replace(['\u{2068}', '\u{2069}'], ""))
            .collect();

        assert_eq!(
            widest,
            ["888 apps", "888 background", "888 services", "888 kernel", "8888 processes"],
            "a count that grows must not push its neighbours"
        );
    }

    #[guinea::test(iterations = 4, exclusive = "store")]
    fn the_wsl_heading_carries_the_notes_on_how_it_is_shown(h: &mut Harness) {
        let _store = start(h);
        let h = &*h;
        let mut page = wsl_page(h);

        let items = page.items();
        let with_notes: Vec<String> = items
            .iter()
            .filter(|item| item.find(ProcessesMark::WslNotes).is_some())
            .map(label)
            .collect();
        assert_eq!(with_notes, ["WSL (1)"]);
    }

    #[guinea::test(iterations = 4, exclusive = "store")]
    fn an_environment_chevron_leaves_the_press_to_its_row_so_it_toggles_once(h: &mut Harness) {
        let _store = start(h);
        let h = &*h;
        let mut page = wsl_page(h);
        let ubuntu = page.item_where(|item| label(item) == "Ubuntu (3)").tree();
        assert!(
            ubuntu.find(ProcessesMark::Chevron).is_none(),
            "a chevron of its own would toggle on top of the row press and undo it"
        );

        select(&mut page, "Ubuntu (3)");
        assert!(labels(&mut page).contains(&"init".to_string()), "{:?}", labels(&mut page));

        select(&mut page, "Ubuntu (3)");
        assert!(!labels(&mut page).contains(&"init".to_string()), "{:?}", labels(&mut page));
    }

    fn first_margin(page: &mut Mounted<'_, Processes>, node: &Node) -> Option<PropertyValue> {
        if let Some(margin) = page.at(node.at).property(PropertyId::Margin) {
            return Some(margin.clone());
        }
        node.children.iter().find_map(|child| first_margin(page, child))
    }

    fn name_inset(page: &mut Mounted<'_, Processes>, wanted: &str) -> PropertyValue {
        let item = page.item_where(|item| label(item) == wanted).tree();
        let cell = item.find(ProcessColumn::Name).expect("a name cell");
        first_margin(page, cell).unwrap_or_else(|| panic!("{cell:#?}"))
    }

    #[guinea::test(iterations = 4, exclusive = "store")]
    fn linux_processes_sit_flush_with_their_environment(h: &mut Harness) {
        let _store = start(h);
        let h = &*h;
        let mut page = wsl_page(h);
        select(&mut page, "Ubuntu (3)");
        open(&mut page, "chrome.exe (3)");
        select(&mut page, "idle.exe");

        let environment = name_inset(&mut page, "Ubuntu (3)");
        assert_eq!(name_inset(&mut page, "bash"), environment);
        assert_eq!(name_inset(&mut page, "notepad.exe"), environment);
        assert_ne!(
            name_inset(&mut page, "chrome.exe"),
            environment,
            "a member of a Windows group is still set in"
        );
    }

    #[guinea::test(iterations = 4, exclusive = "store")]
    fn a_linux_process_is_selected_apart_from_the_windows_process_with_its_pid(h: &mut Harness) {
        let _store = start(h);
        let h = &*h;
        let mut page = wsl_page(h);
        select(&mut page, "Ubuntu (3)");

        right_click(&mut page, "init");
        let state = h.state::<ProcessesState>();
        assert_eq!((state.selected, state.selected_linux), (None, Some(NOTEPAD)));
        assert_eq!(marked_selected(&mut page), ["init"]);
        assert!(!menu_open(&page), "nothing to do to a Linux process yet");
        assert!(!end_task_enabled(&page));
        assert!(
            page.find_text("Selected: \u{2068}init\u{2069} | PID \u{2068}1\u{2069} | \u{2068}Ubuntu\u{2069}").is_some(),
            "{:#?}",
            page.tree()
        );

        select(&mut page, "notepad.exe");
        let state = h.state::<ProcessesState>();
        assert_eq!((state.selected, state.selected_linux), (Some(NOTEPAD), None));
        assert_eq!(marked_selected(&mut page), ["notepad.exe"]);

        select(&mut page, "bash");
        h.publish(PressedAway).settle();
        page.settle();
        let state = h.state::<ProcessesState>();
        assert_eq!((state.selected, state.selected_linux), (None, None));
        assert_eq!(marked_selected(&mut page), Vec::<String>::new());
    }

    #[guinea::test(iterations = 4, exclusive = "store")]
    fn a_lost_linux_agent_leaves_the_vm_heading_alone(h: &mut Harness) {
        let _store = start(h);
        let h = &*h;
        let mut page = wsl_page(h);

        h.publish(RemoteScanResult::Unavailable(AgentConnectionState::Disconnected)).settle();
        page.settle();

        let all = labels(&mut page);
        assert!(all.contains(&"WSL (0)".to_string()), "{all:?}");
        assert!(!all.iter().any(|label| label.starts_with("Ubuntu")), "{all:?}");
    }

    fn shown_columns(page: &mut Mounted<'_, Processes>) -> Vec<ProcessColumn> {
        let row = page.item_where(|item| label(item) == "notepad.exe").tree();
        let mut slots: Vec<(i32, ProcessColumn)> = ProcessColumn::ALL
            .into_iter()
            .filter_map(|column| {
                let cell = row.find(column)?;
                match page.property(cell.at, PropertyId::GridColumn) {
                    Some(PropertyValue::I32(slot)) => Some((*slot, column)),
                    _ => None,
                }
            })
            .collect();
        slots.sort_by_key(|(slot, _)| *slot);
        slots.into_iter().map(|(_, column)| column).collect()
    }

    #[guinea::test(iterations = 4, exclusive = "store")]
    fn a_column_dragged_along_the_header_moves_and_stays_there(h: &mut Harness) {
        use ProcessColumn::*;
        let _store = start(h);
        let h = &*h;
        let mut page = mount(h);
        assert_eq!(shown_columns(&mut page), [Name, Cpu, Memory, Net, Disk]);

        let sorted_by = h.state::<ProcessesState>().sort_column;
        page.drag(Cpu, Drag::by(100.0, 0.0)).settle();
        page.settle();

        assert_eq!(shown_columns(&mut page), [Name, Memory, Cpu, Net, Disk]);
        assert_eq!(h.state::<ProcessesState>().sort_column, sorted_by, "a drag is not a click on the header");
        let ranks = ProcessesSettings::new().unwrap().columns().order();
        assert_eq!(ranks.get("memory"), Some(1));
        assert_eq!(ranks.get("cpu"), Some(2));
    }

    fn sections(page: &mut Mounted<'_, Processes>) -> Vec<String> {
        page.items()
            .iter()
            .filter(|item| item.find(ProcessesMark::SectionGrip).is_some())
            .map(label)
            .collect()
    }

    fn drag_heading(page: &mut Mounted<'_, Processes>, heading: &str, drag: Drag) {
        page.item_where(|item| label(item) == heading)
            .drag(ProcessesMark::SectionGrip, drag)
            .settle();
        page.settle();
    }

    fn apps_open(page: &mut Mounted<'_, Processes>) -> bool {
        labels(page).contains(&"notepad.exe".to_string())
    }

    fn kept_rank(section: ProcessCategory) -> Option<u32> {
        ProcessesSettings::new().unwrap().grouping().section_order().get(&section.id().to_string())
    }

    #[guinea::test(iterations = 4, exclusive = "store")]
    fn a_section_dragged_below_the_rest_goes_last_and_stays_there(h: &mut Harness) {
        let _store = start(h);
        let h = &*h;
        let mut page = mount(h);
        let before = sections(&mut page);
        assert!(before[0].starts_with("Apps"), "{before:?}");

        let apps = before[0].clone();
        drag_heading(&mut page, &apps, Drag::by(0.0, 10_000.0));
        let after = sections(&mut page);
        assert_eq!(after.last(), Some(&apps), "{after:?}");
        assert_eq!(after[..after.len() - 1], before[1..], "{after:?}");
        assert!(page.find(ProcessesMark::DropLine).is_none());
        assert!(apps_open(&mut page), "letting go does not fold the section");
        assert_eq!(kept_rank(ProcessCategory::App), Some(6), "the order is kept in the settings");

        select(&mut page, &apps);
        assert!(!apps_open(&mut page), "a plain click still folds it");
    }

    #[guinea::test(iterations = 4, exclusive = "store")]
    fn a_nudge_or_a_lost_pointer_moves_nothing(h: &mut Harness) {
        let _store = start(h);
        let h = &*h;
        let mut page = mount(h);
        let before = sections(&mut page);
        let apps = before[0].clone();

        drag_heading(&mut page, &apps, Drag::by(0.0, 2.0));
        assert_eq!(sections(&mut page), before);
        assert!(!apps_open(&mut page), "a nudge is a click: it folds");
        select(&mut page, &apps);
        assert!(apps_open(&mut page));

        drag_heading(&mut page, &apps, Drag::by(0.0, 10_000.0).lost());
        assert!(page.find(ProcessesMark::DropLine).is_none(), "a lost pointer shows no gap");
        assert_eq!(sections(&mut page), before);
        assert_eq!(kept_rank(ProcessCategory::App), None);

        select(&mut page, &apps);
        assert!(!apps_open(&mut page), "the next click is a click");
    }

    #[guinea::test(iterations = 4, exclusive = "store")]
    fn a_kept_order_is_there_from_the_start(h: &mut Harness) {
        let _store = start(h);
        ProcessesSettings::new()
            .unwrap()
            .grouping()
            .section_order()
            .insert(ProcessCategory::WindowsKernel.id().to_string(), &0)
            .unwrap();
        let h = &*h;
        let mut page = mount(h);
        report(h, with_kernel());
        page.settle();

        let order = sections(&mut page);
        assert!(order[0].starts_with("Windows kernel"), "{order:?}");
        assert!(order[1].starts_with("Apps"), "{order:?}");
    }
}
