use app_contracts::features::processes::ProcessesState;
use domain::features::processes::settings::ProcessesSettings;
use domain::features::processes::{ProcessesDeps, ProcessesFeature};
use guinea::feature::FeatureInitContext;
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
        collapsed_sections: settings.grouping().collapsed_sections().clone(),
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
        self.0.view(&state, &dispatch, &l10n, palette, forward)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use app_contracts::features::agents::{
        SignatureStatus, WindowsActionRequest, WindowsProcessStats, WindowsReport,
        WindowsReportMessage, WindowsServiceState, WindowsServiceStats,
    };
    use std::time::Duration;

    use app_contracts::features::agent_link::AgentLinkState;
    use app_contracts::features::processes::{ProcessColumn, ProcessesState};
    use domain::features::agent_link::{AgentLinkDeps, AgentLinkFeature};
    use app_contracts::features::services::{ServiceColumn, ServicesState};
    use guinea::core::remote;
    use domain::features::processes::windows_scan::AppWindows;
    use guinea::app::Harness;
    use guinea::winui::harness::{Mounted, Node, PropertyId, PropertyValue};
    use guinea_plugin_l10n::L10nPlugin;
    use guinea_plugin_store::amethystate::store::builder::Backend;
    use guinea_plugin_store::StorePlugin;
    use ui::pages::processes::ProcessesMark;
    use windows_reactor::ColorScheme;

    use super::*;

    const NOTEPAD: u32 = 10;
    const CHROME: [u32; 3] = [21, 22, 23];
    const SVCHOST: u32 = 30;
    const CMD: u32 = 40;
    const CONHOST: u32 = 41;
    const IDLE: u32 = 50;
    const BROKER: u32 = 60;

    thread_local! {
        static NOTEPAD_WINDOW: std::cell::Cell<bool> = const { std::cell::Cell::new(true) };
    }

    fn desktop_windows() -> AppWindows {
        let mut windows = AppWindows::default();
        if NOTEPAD_WINDOW.get() {
            windows.add(NOTEPAD, "Untitled - Notepad".to_string());
        }
        windows
    }

    fn process(pid: u32, name: &str, cpu: f32) -> WindowsProcessStats {
        WindowsProcessStats {
            pid,
            name: name.into(),
            cpu_percent: cpu,
            private_working_set_kb: 1_000 * u64::from(pid),
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

        vec![
            broker,
            process(NOTEPAD, "notepad.exe", 5.0),
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
        h.plugin(StorePlugin::at(dir.path().join("settings")).backend(Backend::Json))
            .unwrap()
            .plugin(L10nPlugin::<app_contracts::l10n::L10n>::new("en"))
            .unwrap()
            .provide(ProcessesDeps { windows: desktop_windows });
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
        assert!(page.find_text("notepad.exe — Exited").is_some(), "{:#?}", page.tree());

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

    #[guinea::test(iterations = 4, exclusive = "store")]
    fn a_tool_sorts_and_selects_through_remote_actions(h: &mut Harness) {
        let _store = start(h);
        let h = &*h;
        let mut page = mount(h);
        let scopes = [h.segment().context().scope.clone()];

        remote::act_in(&scopes, "Terminate", "null").unwrap();
        remote::act_in(&scopes, "Sort", "\"Memory\"").unwrap();
        remote::act_in(&scopes, "Select", "10").unwrap();
        page.settle();

        let state = h.state::<ProcessesState>();
        assert_eq!(state.sort_column, ProcessColumn::Memory);
        assert_eq!(state.selected, Some(NOTEPAD));
    }

    #[guinea::test(iterations = 2, exclusive = "store")]
    fn a_tool_sorts_services_through_remote_actions(h: &mut Harness) {
        let _store = start(h);
        let h = &*h;
        let params = crate::routes::ServicesParams::default();
        let mut page = Mounted::<crate::pages::Services>::mount_with(&h.segment(), params, |page| {
            View::provide(scheme_context(), ColorScheme::Dark, page)
        })
        .unwrap();
        let scopes = [h.segment().context().scope.clone()];

        remote::act_in(&scopes, "Sort", "\"Status\"").unwrap();
        page.settle();

        assert_eq!(h.state::<ServicesState>().sort_column, ServiceColumn::Status);
    }

    #[test]
    fn every_action_and_event_is_open_to_tools() {
        assert_eq!(
            remote::actions(),
            [
                "Command", "Deselect", "Refresh", "RefreshDistros", "Select", "SetOpen", "SetWidth",
                "Sort", "StartInProcess", "Terminate", "Toggle",
            ]
        );
        assert_eq!(
            remote::events(),
            [
                "AgentStateRequest",
                "RemoteScanResult",
                "ScanTick",
                "WindowsActionRequest",
                "WindowsActionResponse",
                "WindowsAgentInProcess",
                "WindowsAgentRuntimeEvent",
                "WindowsReportMessage",
                "WslAgentRuntimeEvent",
            ]
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
}
