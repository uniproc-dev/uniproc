use app_contracts::features::sidebar::{SetOpen, SetWidth, SidebarState};
use domain::features::sidebar::SidebarFeature;
use guinea::feature::FeatureInitContext;
use guinea::winui::{layout, Layout, LayoutCx, UpdateCx, UseNavigate};
use ui::ShellNav;
use windows_reactor::{Callback, View};

use crate::routes::Route;

#[guinea::slot]
pub struct PaneFooter;

#[guinea::slot]
pub struct Overlay;

#[derive(Default)]
pub struct Shell;

pub enum ShellMsg {
    Resize(f64),
    OpenChanged(bool),
}

fn nav(route: &Route) -> ShellNav {
    match route {
        Route::Processes {} | Route::ProcessesSettings {} => ShellNav::Processes,
        Route::Activity {} | Route::ActivityGroups {} => ShellNav::Activity,
        Route::Services {} => ShellNav::Services,
        Route::Wsl {} => ShellNav::Wsl,
        Route::System {} | Route::SystemTools {} => ShellNav::System,
        Route::Settings {} => ShellNav::Settings,
    }
}

#[layout]
impl Layout for Shell {
    type Params = crate::routes::ShellParams;
    type Installs = SidebarFeature;
    type Message = ShellMsg;

    fn install(ctx: &FeatureInitContext, _params: &Self::Params) -> anyhow::Result<Self::Installs> {
        ctx.install(&())
    }

    fn update(&mut self, message: ShellMsg, cx: &mut UpdateCx<'_, Self>) {
        match message {
            ShellMsg::Resize(width) => {
                let (_, dispatch) = cx.read::<SidebarState>();
                dispatch.emit(SetWidth(width.round() as u64));
            }
            ShellMsg::OpenChanged(open) => {
                let (sidebar, dispatch) = cx.read::<SidebarState>();
                if sidebar.open != open {
                    dispatch.emit(SetOpen(open));
                }
            }
        }
    }

    fn view(&self, cx: &mut LayoutCx<'_, '_, Self>) -> View {
        let (sidebar, _) = cx.read::<SidebarState>();
        let l10n = ui::l10n::use_tr(cx);
        let navigate = cx.use_navigate::<Route>();

        let children = cx.child_routes::<Route>();
        let menu = children.iter().map(|child| (nav(&child.route), child.current)).collect();
        let targets: Vec<(ShellNav, Route)> = children.into_iter().map(|child| (nav(&child.route), child.route)).collect();
        let on_select = Callback::new(move |picked: ShellNav| {
            if let Some((_, route)) = targets.iter().find(|(nav, _)| *nav == picked) {
                navigate.to(route.clone());
            }
        });

        ui::shell_view(ui::ShellProps {
            l10n: &l10n,
            open: sidebar.open,
            width: sidebar.width as f64,
            menu,
            content: cx.outlet(),
            pane_footer: cx.slot::<PaneFooter>(),
            overlay: cx.slot::<Overlay>(),
            on_select,
            on_resize: cx.on(ShellMsg::Resize),
            on_open_changed: cx.on(ShellMsg::OpenChanged),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::time::Duration;

    use guinea::app::Harness;
    use guinea::core::actor::event_bus::{RpcRequest, RpcResponse};
    use guinea::prelude::GlobalEventBus;
    use guinea::winui::harness::Mounted;
    use domain::features::agents::providers::windows::SERVICE_DISPLAY_NAME;
    use domain::features::agents::settings::AgentSettings;
    use domain::features::system::SystemDeps;
    use guinea_plugin_store::StoreAccess;
    use uuid::Uuid;

    use std::sync::Arc;

    use app_contracts::features::activity::{ActivityState, Clock, Hover};
    use app_contracts::features::agent_link::{AgentLinkState, InProcess};
    use app_contracts::features::agents::{ProcessCame, ProcessEvent, ProcessInstance, WindowsProcessEvents};
    use domain::features::activity::ActivityDeps;
    use app_contracts::features::settings::{SettingsState, SidebarChart};
    use app_contracts::features::agents::{
        ActionOutcome, AgentConnectionState, WindowsAction, WindowsActionRequest, WindowsMachineSample,
        WindowsMachineStats, WindowsReport, WindowsReportMessage,
    };

    use super::*;
    use crate::pages::Services;
    use crate::test_agent;

    fn mount(h: &Harness) -> Mounted<'_, Route> {
        Mounted::routed(h, Route::Services {}).unwrap()
    }

    fn splash_shown(page: &Mounted<'_, Route>) -> bool {
        page.find(ui::SplashMark::Splash).is_some()
    }

    fn content_shown(page: &Mounted<'_, Route>) -> bool {
        !splash_shown(page) && page.is_mounted::<Services>()
    }

    fn agent(page: &Mounted<'_, Route>) -> AgentConnectionState {
        page.state::<AgentLinkState>().windows
    }

    fn start(h: &mut Harness, agent_up: bool) {
        test_agent::reset(agent_up);
        h.provide(ActivityDeps {
            now: || 1000 * HOUR + HOUR - TICK,
            clock: |_| Clock::default(),
        })
        .install_application_with(crate::app::with_fakes)
        .unwrap()
        .provide(test_agent::agent_link())
            .provide(SystemDeps {
                locate: |_| None,
                launch: |_| {},
            });
    }

    fn answers_to_a_kill(h: &Harness, page: &mut Mounted<'_, Route>) -> Vec<ActionOutcome> {
        let answered = Rc::new(RefCell::new(Vec::new()));
        let heard = answered.clone();
        let _watch = GlobalEventBus::subscribe_fn(move |RpcResponse { payload, .. }: RpcResponse<ActionOutcome>| {
            heard.borrow_mut().push(payload)
        });
        h.publish(RpcRequest {
            correlation_id: Uuid::new_v4(),
            payload: WindowsActionRequest(WindowsAction::Kill { pid: 42 }),
            chain: Vec::new(),
        })
        .settle();
        after(h, page, 1);
        answered.take()
    }

    #[guinea::test(iterations = 4, exclusive = "agent")]
    fn an_action_with_no_service_and_no_monitor_in_process_says_not_connected(h: &mut Harness) {
        start(h, false);
        let h = &*h;
        let mut page = mount(h);

        assert_eq!(answers_to_a_kill(h, &mut page), [ActionOutcome::NotConnected]);
        assert!(test_agent::in_process_actions().is_empty());
    }

    fn after(h: &Harness, page: &mut Mounted<'_, Route>, seconds: u64) {
        h.advance(Duration::from_secs(seconds));
        page.settle();
    }

    #[guinea::test(iterations = 16, exclusive = "agent")]
    fn the_splash_gives_up_after_five_attempts_and_keeps_trying(h: &mut Harness) {
        start(h, false);
        let h = &*h;
        let mut page = mount(h);
        assert!(splash_shown(&page), "{:#?}", page.tree());
        assert_eq!(test_agent::connects(), 1);

        after(h, &mut page, 9);
        assert_eq!(test_agent::connects(), 4);
        assert_ne!(agent(&page), AgentConnectionState::GaveUp);
        assert!(splash_shown(&page), "{:#?}", page.tree());
        assert!(page.find(ui::SplashMark::Unreachable).is_none(), "not given up yet");

        after(h, &mut page, 3);
        assert_eq!(test_agent::connects(), 5, "one attempt per three-second window");
        assert_eq!(agent(&page), AgentConnectionState::GaveUp);
        assert!(splash_shown(&page), "{:#?}", page.tree());
        assert!(page.find(ui::SplashMark::Unreachable).is_some(), "{:#?}", page.tree());

        after(h, &mut page, 3);
        assert_eq!(test_agent::connects(), 6, "giving up does not stop the attempts");
        assert_eq!(agent(&page), AgentConnectionState::GaveUp);

        test_agent::set_up(true);
        after(h, &mut page, 3);
        assert!(!splash_shown(&page), "{:#?}", page.tree());
        assert!(content_shown(&page), "{:#?}", page.tree());
    }

    #[guinea::test(iterations = 8, exclusive = "agent")]
    fn a_shorter_ping_interval_takes_effect_at_once(h: &mut Harness) {
        start(h, true);
        let h = &*h;
        let mut page = mount(h);
        let interval = h.segment().settings::<AgentSettings>().ping_interval_ms();
        interval.set(60_000);
        after(h, &mut page, 5);
        let before = test_agent::pings();

        interval.set(1_000);
        after(h, &mut page, 2);

        assert!(test_agent::pings() > before, "a minute-long wait is not sat out after the interval shrank");
    }

    #[guinea::test(iterations = 4, exclusive = "agent")]
    fn the_settings_item_opens_the_settings_page(h: &mut Harness) {
        start(h, true);
        let h = &*h;
        let mut page = mount(h);

        page.click_text("Settings").settle();
        page.settle();

        assert_eq!(page.route(), Route::Settings {});
    }

    #[guinea::test(iterations = 4, exclusive = "agent")]
    fn the_activity_item_opens_the_activity_page(h: &mut Harness) {
        start(h, true);
        let h = &*h;
        let mut page = mount(h);
        assert!(page.find_text("Activity").is_some(), "the sidebar lists Activity");

        page.click_text("Activity").settle();
        page.settle();

        assert_eq!(page.route(), Route::Activity {});
    }

    #[guinea::test(iterations = 4, exclusive = "agent")]
    fn the_system_item_opens_the_system_page(h: &mut Harness) {
        start(h, true);
        let h = &*h;
        let mut page = mount(h);

        page.click_text("System").settle();
        page.settle();

        assert_eq!(page.route(), Route::System {});
    }

    fn unreachable_line(page: &Mounted<'_, Route>) -> Option<String> {
        page.tree()
            .find(ui::SplashMark::Unreachable)
            .and_then(|node| node.text.clone())
    }

    #[guinea::test(iterations = 8, exclusive = "agent")]
    fn the_splash_names_the_service_it_cannot_reach(h: &mut Harness) {
        start(h, false);
        let h = &*h;
        let mut page = mount(h);

        after(h, &mut page, 9);
        assert_ne!(agent(&page), AgentConnectionState::GaveUp);
        assert_eq!(unreachable_line(&page), None, "slow is not unreachable yet");

        after(h, &mut page, 3);
        assert_eq!(agent(&page), AgentConnectionState::GaveUp);
        let line = unreachable_line(&page).unwrap_or_else(|| panic!("{:#?}", page.tree()));
        assert_eq!(
            line,
            format!("Can’t connect to the “\u{2068}{SERVICE_DISPLAY_NAME}\u{2069}” service")
        );

        after(h, &mut page, 3);
        assert!(unreachable_line(&page).is_some(), "still named while it keeps trying");

        test_agent::set_up(true);
        after(h, &mut page, 3);
        assert_eq!(unreachable_line(&page), None, "{:#?}", page.tree());
    }

    #[guinea::test(iterations = 8, exclusive = "agent")]
    fn the_splash_says_the_service_is_older_than_uniproc_and_keeps_trying(h: &mut Harness) {
        start(h, false);
        test_agent::set_outdated(true);
        let h = &*h;
        let mut page = mount(h);

        after(h, &mut page, 6);
        assert_eq!(agent(&page), AgentConnectionState::Outdated);
        let line = page
            .tree()
            .find(ui::SplashMark::Outdated)
            .and_then(|node| node.text.clone())
            .unwrap_or_else(|| panic!("{:#?}", page.tree()));
        assert_eq!(
            line,
            format!("The “\u{2068}{SERVICE_DISPLAY_NAME}\u{2069}” service is older than this Uniproc. Update it.")
        );
        assert!(page.find(ui::SplashMark::Unreachable).is_none(), "it answered; it is not unreachable");
        assert!(test_agent::connects() >= 2, "still tried: {}", test_agent::connects());

        test_agent::set_outdated(false);
        test_agent::set_up(true);
        after(h, &mut page, 3);
        assert!(!splash_shown(&page), "an updated service is picked up: {:#?}", page.tree());
    }

    #[guinea::test(iterations = 8, exclusive = "agent")]
    fn a_service_that_drops_every_connection_at_once_is_not_hammered(h: &mut Harness) {
        start(h, true);
        test_agent::set_drops(true);
        let h = &*h;
        let mut page = mount(h);

        after(h, &mut page, 9);
        assert!(
            test_agent::connects() <= 4,
            "one connection per three-second window, not a reconnect loop: {}",
            test_agent::connects()
        );
        assert!(test_agent::connects() >= 3, "still reconnecting: {}", test_agent::connects());
    }

    #[guinea::test(iterations = 4, exclusive = "agent")]
    fn the_splash_is_gone_as_soon_as_the_agent_answers(h: &mut Harness) {
        start(h, true);
        let h = &*h;
        let page = mount(h);

        assert!(content_shown(&page), "{:#?}", page.tree());
        assert_eq!(test_agent::connects(), 1);
    }

    fn in_process(page: &Mounted<'_, Route>) -> InProcess {
        page.state::<AgentLinkState>().in_process
    }

    fn in_process_error(page: &Mounted<'_, Route>) -> Option<String> {
        page.tree()
            .find(ui::SplashMark::InProcessError)
            .and_then(|node| node.text.clone())
    }

    fn start_in_process(page: &mut Mounted<'_, Route>) {
        let _ = page.click(ui::SplashMark::OpenInProcess);
        page.settle();
    }

    #[guinea::test(iterations = 4, exclusive = "agent")]
    fn the_splash_offers_the_monitor_in_process_after_five_seconds(h: &mut Harness) {
        start(h, false);
        let h = &*h;
        let mut page = mount(h);

        after(h, &mut page, 4);
        assert!(page.find(ui::SplashMark::OpenInProcess).is_none(), "not offered yet");
        assert!(page.find_text("Starting is taking longer than usual").is_none());

        after(h, &mut page, 1);
        assert!(page.find_text("Starting is taking longer than usual").is_some(), "{:#?}", page.tree());
        assert!(page.find_text("Open monitor in process").is_some(), "{:#?}", page.tree());
        assert_eq!(test_agent::in_process_starts(), 0, "offered, not started");
        assert_eq!(in_process(&page), InProcess::Off);
    }

    #[guinea::test(iterations = 8, exclusive = "agent")]
    fn the_monitor_in_process_takes_over_from_the_service(h: &mut Harness) {
        start(h, false);
        test_agent::set_elevated(true);
        let h = &*h;
        let mut page = mount(h);

        after(h, &mut page, 5);
        start_in_process(&mut page);

        assert_eq!(test_agent::in_process_starts(), 1);
        assert_eq!(in_process(&page), InProcess::Running);
        assert_eq!(agent(&page), AgentConnectionState::Connected);
        assert!(!splash_shown(&page), "{:#?}", page.tree());
        assert!(content_shown(&page), "{:#?}", page.tree());

        let connects = test_agent::connects();
        after(h, &mut page, 15);
        assert_eq!(test_agent::connects(), connects, "the service is left alone");
        assert_eq!(agent(&page), AgentConnectionState::Connected, "no give-up from the dormant service");

        test_agent::set_up(true);
        after(h, &mut page, 5);
        assert_eq!(test_agent::connects(), connects, "not even once it is up");

        let reports = test_agent::in_process_reports();
        after(h, &mut page, 2);
        assert!(test_agent::in_process_reports() >= reports + 3, "the in-process agent reports without a tick");

        assert_eq!(answers_to_a_kill(h, &mut page), [ActionOutcome::Done], "the in-process agent answers, once");
        assert!(
            matches!(test_agent::in_process_actions().as_slice(), [WindowsAction::Kill { pid: 42 }]),
            "{:?}",
            test_agent::in_process_actions()
        );
    }

    #[guinea::test(iterations = 8, exclusive = "agent")]
    fn without_admin_rights_the_button_restarts_uniproc_elevated(h: &mut Harness) {
        start(h, false);
        let h = &*h;
        let mut page = mount(h);
        assert!(!page.state::<AgentLinkState>().elevated);

        after(h, &mut page, 5);
        start_in_process(&mut page);

        assert_eq!(test_agent::relaunches(), 1);
        assert_eq!(test_agent::in_process_starts(), 0, "the elevated copy starts it, not this one");
        assert_eq!(test_agent::closes(), 1, "this copy makes way for the elevated one");
        assert_eq!(in_process(&page), InProcess::Elevating);
        assert_eq!(in_process_error(&page), None, "{:#?}", page.tree());
    }

    #[guinea::test(iterations = 8, exclusive = "agent")]
    fn a_refused_elevation_leaves_uniproc_as_it_was(h: &mut Harness) {
        start(h, false);
        test_agent::set_relaunch_refused(true);
        let h = &*h;
        let mut page = mount(h);

        after(h, &mut page, 5);
        start_in_process(&mut page);

        assert_eq!(test_agent::relaunches(), 1);
        assert_eq!(test_agent::closes(), 0);
        assert_eq!(in_process(&page), InProcess::Off);
        assert_eq!(in_process_error(&page), None, "{:#?}", page.tree());
        assert!(splash_shown(&page), "{:#?}", page.tree());

        start_in_process(&mut page);
        assert_eq!(test_agent::relaunches(), 2, "the button stays usable after a refusal");
    }

    #[guinea::test(iterations = 8, exclusive = "agent")]
    fn an_elevated_restart_opens_the_monitor_in_process_at_once(h: &mut Harness) {
        start(h, false);
        test_agent::set_elevated(true);
        test_agent::set_asked_at_start(true);
        let h = &*h;
        let mut page = mount(h);
        page.settle();

        assert!(page.state::<AgentLinkState>().elevated);
        assert_eq!(test_agent::in_process_starts(), 1, "no click and no five-second wait");
        assert_eq!(in_process(&page), InProcess::Running);
        assert!(!splash_shown(&page), "{:#?}", page.tree());
        assert_eq!(test_agent::relaunches(), 0);
    }

    #[guinea::test(iterations = 4, exclusive = "agent")]
    fn a_copy_asked_to_start_it_but_not_elevated_does_not_restart_again(h: &mut Harness) {
        start(h, false);
        test_agent::set_asked_at_start(true);
        let h = &*h;
        let mut page = mount(h);
        after(h, &mut page, 1);

        assert_eq!(test_agent::relaunches(), 0);
        assert_eq!(test_agent::closes(), 0);
        assert_eq!(test_agent::in_process_starts(), 0);
        assert_eq!(in_process(&page), InProcess::Off);
    }

    #[guinea::test(iterations = 8, exclusive = "agent")]
    fn a_monitor_in_process_that_will_not_start_hands_back_to_the_service(h: &mut Harness) {
        start(h, false);
        test_agent::set_elevated(true);
        test_agent::set_in_process_fails(true);
        let h = &*h;
        let mut page = mount(h);

        after(h, &mut page, 5);
        start_in_process(&mut page);
        assert_eq!(in_process(&page), InProcess::Failed);
        assert!(splash_shown(&page), "{:#?}", page.tree());

        let connects = test_agent::connects();
        after(h, &mut page, 3);
        assert!(test_agent::connects() > connects, "the service is tried again");

        test_agent::set_up(true);
        after(h, &mut page, 3);
        assert_eq!(agent(&page), AgentConnectionState::Connected);
        assert!(!splash_shown(&page), "{:#?}", page.tree());
    }

    #[guinea::test(iterations = 4, exclusive = "agent")]
    fn the_service_still_wins_after_the_elevation_was_refused(h: &mut Harness) {
        start(h, false);
        test_agent::set_relaunch_refused(true);
        let h = &*h;
        let mut page = mount(h);

        after(h, &mut page, 5);
        start_in_process(&mut page);
        assert_eq!(in_process(&page), InProcess::Off);

        test_agent::set_up(true);
        after(h, &mut page, 3);
        assert_eq!(agent(&page), AgentConnectionState::Connected);
        assert!(!splash_shown(&page), "{:#?}", page.tree());
    }

    fn open_pane(page: &mut Mounted<'_, Route>) {
        page.dispatch::<SidebarState>().emit(SetOpen(true));
        page.settle();
    }

    fn tile_shown(page: &Mounted<'_, Route>, chart: SidebarChart) -> bool {
        page.find(ui::SidebarMark::tile(chart)).is_some()
    }

    #[guinea::test(iterations = 4, exclusive = "agent")]
    fn a_hidden_chart_leaves_the_pane_and_comes_back(h: &mut Harness) {
        start(h, true);
        let h = &*h;
        let mut page = mount(h);
        open_pane(&mut page);
        for chart in SidebarChart::ALL {
            assert!(tile_shown(&page, chart), "{chart:?}: {:#?}", page.tree());
        }
        assert!(page.find(ui::SidebarMark::Charts).is_some(), "the menu that picks them");

        page.click(ui::SidebarMark::show(SidebarChart::Disk)).settle();
        page.settle();
        assert!(!tile_shown(&page, SidebarChart::Disk), "{:#?}", page.tree());
        assert!(tile_shown(&page, SidebarChart::Network));
        assert!(!page.state::<SettingsState>().sidebar_charts.shows(SidebarChart::Disk));

        page.click(ui::SidebarMark::show(SidebarChart::Disk)).settle();
        page.settle();
        assert!(tile_shown(&page, SidebarChart::Disk), "{:#?}", page.tree());
        assert!(page.state::<SettingsState>().sidebar_charts.shows(SidebarChart::Disk));
    }

    #[guinea::test(iterations = 4, exclusive = "agent")]
    fn rate_charts_name_the_top_of_their_scale(h: &mut Harness) {
        start(h, true);
        let h = &*h;
        let mut page = mount(h);
        open_pane(&mut page);
        after(h, &mut page, 1);

        let tree = page.tree();
        let scale = |chart: SidebarChart, top: &str| {
            tree.find(ui::SidebarMark::tile(chart))
                .unwrap_or_else(|| panic!("{chart:?}: {tree:#?}"))
                .find_text(top)
                .is_some()
        };
        assert!(scale(SidebarChart::Disk, "100 KB/s"), "{tree:#?}");
        assert!(scale(SidebarChart::Network, "100 Kbps"), "{tree:#?}");
        assert!(!scale(SidebarChart::Cpu, "100 KB/s"), "a percent chart needs no scale");
    }

    #[guinea::test(iterations = 4, exclusive = "agent")]
    fn the_cpu_chart_names_its_frequency_where_rate_charts_name_their_scale(h: &mut Harness) {
        start(h, true);
        let h = &*h;
        let mut page = mount(h);
        open_pane(&mut page);
        after(h, &mut page, 1);
        h.publish(WindowsReportMessage::Report(std::sync::Arc::new(WindowsReport {
            machine: WindowsMachineStats {
                cpu_current_mhz: 3_800,
                cpu_max_mhz: 2_900,
                ..WindowsMachineStats::default()
            },
            ..WindowsReport::default()
        })))
        .settle();
        page.settle();

        let tree = page.tree();
        let cpu = tree
            .find(ui::SidebarMark::tile(SidebarChart::Cpu))
            .unwrap_or_else(|| panic!("{tree:#?}"));
        fn texts(node: &guinea::winui::harness::Node, out: &mut Vec<String>) {
            out.extend(node.text.as_deref().map(|text| text.replace(['\u{2068}', '\u{2069}'], "")));
            for child in &node.children {
                texts(child, out);
            }
        }
        let mut shown = Vec::new();
        texts(cpu, &mut shown);
        assert!(shown.contains(&"3.8 GHz".to_string()), "{shown:?}");
        assert!(!shown.iter().any(|text| text.contains('/') || text.contains("2.9")), "{shown:?}");
    }

    #[guinea::test(iterations = 4, exclusive = "agent")]
    fn disk_and_network_rates_come_from_the_machine_samples(h: &mut Harness) {
        start(h, true);
        let h = &*h;
        let mut page = mount(h);
        open_pane(&mut page);

        let sample = |clock_100ns, bytes| WindowsMachineSample {
            machine: std::sync::Arc::new(WindowsMachineStats {
                disk_read_bytes: bytes,
                net_rx_bytes: bytes / 2,
                ..WindowsMachineStats::default()
            }),
            clock_100ns,
        };
        h.publish(sample(0, 0));
        h.publish(sample(10_000_000, 2 << 20));
        page.settle();

        let tree = page.tree();
        let reads = |chart: SidebarChart, rate: &str| {
            tree.find(ui::SidebarMark::tile(chart))
                .unwrap_or_else(|| panic!("{chart:?}: {tree:#?}"))
                .find_text(rate)
                .is_some()
        };
        assert!(reads(SidebarChart::Disk, "2.0 MB/s"), "{tree:#?}");
        assert!(reads(SidebarChart::Network, "8.4 Mbps"), "{tree:#?}");
    }

    const TICK: u64 = 10_000_000;
    const HOUR: u64 = 3600 * TICK;

    fn a_start() -> WindowsProcessEvents {
        WindowsProcessEvents {
            history_from: Some(1000 * HOUR),
            events: Arc::from([ProcessEvent::Came(ProcessCame {
                instance: ProcessInstance { pid: 20, sequence: 9 },
                at: 1000 * HOUR + 600 * TICK,
                image_path: r"C:\Tools\tool.exe".into(),
                ..ProcessCame::default()
            })]),
            lost: 0,
        }
    }

    fn logged(h: &Harness) -> usize {
        h.act::<ActivityState>(Hover(Some(ProcessInstance { pid: 20, sequence: 9 })))
            .settle();
        usize::from(h.state::<ActivityState>().hovered.is_some())
    }

    #[guinea::test(iterations = 4, exclusive = "agent")]
    fn starts_the_service_tells_reach_the_activity_log(h: &mut Harness) {
        start(h, true);
        let h = &*h;
        let mut page = mount(h);
        after(h, &mut page, 1);
        assert_eq!(logged(h), 0);

        test_agent::tell(a_start());
        after(h, &mut page, 1);

        assert_eq!(logged(h), 1);
    }

    #[guinea::test(iterations = 4, exclusive = "agent")]
    fn starts_the_monitor_in_process_tells_reach_the_activity_log(h: &mut Harness) {
        start(h, false);
        test_agent::set_elevated(true);
        let h = &*h;
        let mut page = mount(h);
        after(h, &mut page, 5);
        start_in_process(&mut page);

        test_agent::tell(a_start());
        after(h, &mut page, 1);

        assert_eq!(logged(h), 1);
    }
}
