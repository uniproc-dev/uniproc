use app_contracts::features::agent_link::{AgentLinkState, StartInProcess};
use app_contracts::features::agents::AgentConnectionState;
use app_contracts::features::metrics::MetricsState;
use app_contracts::features::settings::{AppTheme, SettingsState, ShowSidebarChart};
use app_contracts::features::sidebar::{SetOpen, SetWidth, SidebarState};
use domain::features::agent_link::{AgentLinkDeps, AgentLinkFeature};
use domain::features::agents::providers::windows::SERVICE_DISPLAY_NAME;
use domain::features::metrics::MetricsFeature;
use domain::features::settings::SettingsFeature;
use domain::features::sidebar::SidebarFeature;
use guinea::feature::FeatureInitContext;
use guinea::prelude::Dispatch;
use ui::l10n::L10n;
use ui::theme::Palette;
use guinea::winui::{layout, Layout, LayoutCx, UpdateCx, UseNavigate, UseRoute};
use guinea_widgets::chart::Chart;
use windows_reactor::{Callback, ColorScheme, View, WindowBackdrop, WindowTheme, WindowVisuals};

use crate::layouts::{ProcessesArea, SystemArea};
use crate::pages::{Services, Settings, Wsl};
use crate::route_memory;
use crate::routes::Route;

const WINDOW_WIDTH: f64 = 1000.0;
const WINDOW_HEIGHT: f64 = 700.0;

pub(crate) fn splash(
    link: &AgentLinkState,
    dispatch: &Dispatch,
    l10n: &L10n,
    palette: Palette,
) -> Option<View> {
    if !link.awaiting_first_connection() {
        return None;
    }
    let dispatch = dispatch.clone();
    Some(ui::splash_view(ui::SplashProps {
        l10n,
        palette,
        in_process_offered: link.in_process_offered,
        in_process: link.in_process,
        service_trouble: match link.windows {
            AgentConnectionState::GaveUp => Some(ui::ServiceTrouble::Unreachable(SERVICE_DISPLAY_NAME)),
            AgentConnectionState::Outdated => Some(ui::ServiceTrouble::Outdated(SERVICE_DISPLAY_NAME)),
            _ => None,
        },
        on_start_in_process: Callback::new(move |()| dispatch.emit(StartInProcess)),
    }))
}

#[derive(Default)]
pub struct ShellLayout {
    scheme: ColorScheme,
    charts: [Chart; 5],
    routes: Option<route_memory::RouteSettings>,
}

fn window_theme(theme: AppTheme) -> WindowTheme {
    match theme {
        AppTheme::System => WindowTheme::System,
        AppTheme::Light => WindowTheme::Light,
        AppTheme::Dark => WindowTheme::Dark,
    }
}

fn icon_theme(scheme: ColorScheme) -> guicons::Theme {
    match scheme {
        ColorScheme::Dark => guicons::Theme::Dark,
        ColorScheme::Light => guicons::Theme::Light,
    }
}

pub enum ShellMsg {
    Scheme(ColorScheme),
    Resize(f64),
    OpenChanged(bool),
}

#[layout]
impl Layout for ShellLayout {
    type Params = crate::routes::ShellLayoutParams;
    type Installs = (SidebarFeature, MetricsFeature, AgentLinkFeature, SettingsFeature);
    type Message = ShellMsg;

    fn install(ctx: &FeatureInitContext, _params: &Self::Params) -> anyhow::Result<Self::Installs> {
        let link = ctx.require_or_default::<AgentLinkDeps>();
        Ok((ctx.install(&())?, ctx.install(&())?, ctx.install(&link)?, ctx.install(&())?))
    }

    fn init(ctx: &FeatureInitContext, _params: &Self::Params) -> Self {
        crate::xaml_resources::override_navigation_view_resources();
        crate::window_press::install();
        let shell = Self {
            routes: route_memory::open(ctx),
            ..Self::default()
        };
        guicons::set_theme(icon_theme(shell.scheme));
        shell
    }

    fn update(&mut self, message: ShellMsg, cx: &mut UpdateCx<'_, Self>) {
        match message {
            ShellMsg::Scheme(scheme) => {
                self.scheme = scheme;
                guicons::set_theme(icon_theme(scheme));
            }
            ShellMsg::Resize(width) => {
                let (_, dispatch) = cx.read::<SidebarState, _>();
                dispatch.emit(SetWidth(width.round() as u64));
            }
            ShellMsg::OpenChanged(open) => {
                let (sidebar, dispatch) = cx.read::<SidebarState, _>();
                if sidebar.open != open {
                    dispatch.emit(SetOpen(open));
                }
            }
        }
    }

    fn view(&self, cx: &mut LayoutCx<'_, Self>) -> View {
        let on_scheme = cx.on(ShellMsg::Scheme);
        cx.on_color_scheme(on_scheme);
        let (settings, settings_dispatch) = cx.read::<SettingsState, _>();
        cx.window_visuals(
            WindowVisuals::new()
                .client_size(WINDOW_WIDTH, WINDOW_HEIGHT)
                .backdrop(WindowBackdrop::Mica)
                .theme(window_theme(settings.theme)),
        );

        let current = cx.use_route::<Route>();
        let routes = self.routes.clone();
        cx.use_effect("uniproc::remember_route", current.clone(), move || {
            if let Some(routes) = &routes {
                route_memory::remember(routes, &current);
            }
            None
        });

        let (sidebar, _) = cx.read::<SidebarState, _>();
        let (metrics, _) = cx.read::<MetricsState, _>();
        let (link, link_dispatch) = cx.read::<AgentLinkState, _>();
        let l10n = ui::l10n::use_tr(cx);
        let nav = cx.use_navigate::<Route>();

        let selected_tag = if cx.child_is::<Services>() {
            "services"
        } else if cx.child_is::<Wsl>() {
            "wsl"
        } else if cx.child_is::<ProcessesArea>() {
            "processes"
        } else if cx.child_is::<SystemArea>() {
            "system"
        } else if cx.child_is::<Settings>() {
            "settings"
        } else {
            ""
        };

        let on_select = Callback::new(move |tag: Option<String>| match tag.as_deref() {
            Some("processes") => nav.to(Route::Processes {}),
            Some("services") => nav.to(Route::Services {}),
            Some("wsl") => nav.to(Route::Wsl {}),
            Some("system") => nav.to(Route::System {}),
            Some("settings") => nav.to(Route::Settings {}),
            _ => {}
        });
        let on_resize = cx.on(ShellMsg::Resize);
        let on_open_changed = cx.on(ShellMsg::OpenChanged);
        let content = View::provide(ui::theme::scheme_context(), self.scheme, cx.outlet());

        let palette = ui::theme::Palette::of(self.scheme);
        ui::shell_view(ui::ShellProps {
            l10n: &l10n,
            palette,
            open: sidebar.open,
            width: sidebar.width as f64,
            selected_tag,
            content,
            splash: splash(&link, &link_dispatch, &l10n, palette),
            metrics: &metrics,
            units: settings.units,
            cadence_ms: settings.update_interval_ms,
            charts: &self.charts,
            shown: settings.sidebar_charts,
            on_show_chart: Callback::new(move |(chart, shown)| settings_dispatch.emit(ShowSidebarChart(chart, shown))),
            on_select,
            on_resize,
            on_open_changed,
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
    use guinea::winui::harness::{Mounted, Outlet};
    use guinea_plugin_l10n::L10nPlugin;
    use domain::features::agents::settings::AgentSettings;
    use guinea_plugin_store::{StoreAccess, StorePlugin};
    use uuid::Uuid;

    use app_contracts::features::agent_link::InProcess;
    use app_contracts::features::settings::SidebarChart;
    use app_contracts::features::agents::{
        ActionOutcome, AgentConnectionState, WindowsAction, WindowsActionRequest, WindowsMachineSample,
        WindowsMachineStats, WindowsReport, WindowsReportMessage,
    };

    use super::*;
    use crate::test_agent;

    fn mount(h: &Harness) -> Mounted<'_, ShellLayout> {
        Mounted::<ShellLayout>::mount_at(h.segment(), crate::routes::ShellLayoutParams::default(), Route::Processes {})
            .unwrap()
    }

    fn splash_shown(page: &Mounted<'_, ShellLayout>) -> bool {
        page.find(ui::SplashMark::Splash).is_some()
    }

    fn content_shown(page: &Mounted<'_, ShellLayout>) -> bool {
        page.find(Outlet).is_some()
    }

    fn agent(h: &Harness) -> AgentConnectionState {
        h.state::<AgentLinkState>().windows
    }

    fn start(h: &mut Harness, agent_up: bool) {
        test_agent::reset(agent_up);
        h.plugin(StorePlugin::in_memory())
            .unwrap()
            .plugin(L10nPlugin::<app_contracts::l10n::L10n>::new("en"))
            .unwrap()
            .feature(test_agent::FakeAgentFeature)
            .unwrap()
            .provide(AgentLinkDeps {
                start_in_process: test_agent::start_in_process,
            });
    }

    fn answers_to_a_kill(h: &Harness, page: &mut Mounted<'_, ShellLayout>) -> Vec<ActionOutcome> {
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

    fn after(h: &Harness, page: &mut Mounted<'_, ShellLayout>, seconds: u64) {
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
        assert_ne!(agent(h), AgentConnectionState::GaveUp);
        assert!(splash_shown(&page), "{:#?}", page.tree());
        assert!(page.find(ui::SplashMark::Unreachable).is_none(), "not given up yet");

        after(h, &mut page, 3);
        assert_eq!(test_agent::connects(), 5, "one attempt per three-second window");
        assert_eq!(agent(h), AgentConnectionState::GaveUp);
        assert!(splash_shown(&page), "{:#?}", page.tree());
        assert!(page.find(ui::SplashMark::Unreachable).is_some(), "{:#?}", page.tree());

        after(h, &mut page, 3);
        assert_eq!(test_agent::connects(), 6, "giving up does not stop the attempts");
        assert_eq!(agent(h), AgentConnectionState::GaveUp);

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
        let interval = h.segment().settings::<AgentSettings>().unwrap().ping_interval_ms();
        interval.set(60_000).unwrap();
        after(h, &mut page, 5);
        let before = test_agent::pings();

        interval.set(1_000).unwrap();
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

        assert_eq!(page.navigated::<Route>(), [Route::Settings {}]);
    }

    #[guinea::test(iterations = 4, exclusive = "agent")]
    fn the_system_item_opens_the_system_page(h: &mut Harness) {
        start(h, true);
        let h = &*h;
        let mut page = mount(h);

        page.click_text("System").settle();
        page.settle();

        assert_eq!(page.navigated::<Route>(), [Route::System {}]);
    }

    fn unreachable_line(page: &Mounted<'_, ShellLayout>) -> Option<String> {
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
        assert_ne!(agent(h), AgentConnectionState::GaveUp);
        assert_eq!(unreachable_line(&page), None, "slow is not unreachable yet");

        after(h, &mut page, 3);
        assert_eq!(agent(h), AgentConnectionState::GaveUp);
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
        assert_eq!(agent(h), AgentConnectionState::Outdated);
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

    fn in_process(h: &Harness) -> InProcess {
        h.state::<AgentLinkState>().in_process
    }

    fn in_process_error(page: &Mounted<'_, ShellLayout>) -> Option<String> {
        page.tree()
            .find(ui::SplashMark::InProcessError)
            .and_then(|node| node.text.clone())
    }

    fn start_in_process(page: &mut Mounted<'_, ShellLayout>) {
        page.click(ui::SplashMark::OpenInProcess).settle();
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
        assert_eq!(in_process(h), InProcess::Off);
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
        assert_eq!(in_process(h), InProcess::Running);
        assert_eq!(agent(h), AgentConnectionState::Connected);
        assert!(!splash_shown(&page), "{:#?}", page.tree());
        assert!(content_shown(&page), "{:#?}", page.tree());

        let connects = test_agent::connects();
        after(h, &mut page, 15);
        assert_eq!(test_agent::connects(), connects, "the service is left alone");
        assert_eq!(agent(h), AgentConnectionState::Connected, "no give-up from the dormant service");

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
    fn the_monitor_in_process_needs_an_elevated_uniproc(h: &mut Harness) {
        start(h, false);
        let h = &*h;
        let mut page = mount(h);

        after(h, &mut page, 5);
        assert_eq!(in_process_error(&page), None);
        start_in_process(&mut page);

        assert_eq!(test_agent::in_process_starts(), 1);
        assert_eq!(in_process(h), InProcess::NotElevated);
        assert!(splash_shown(&page), "{:#?}", page.tree());
        assert_eq!(
            in_process_error(&page).as_deref(),
            Some("Monitoring in process needs Uniproc to run as administrator"),
            "{:#?}",
            page.tree()
        );

        let connects = test_agent::connects();
        after(h, &mut page, 3);
        assert_eq!(test_agent::connects(), connects + 1, "the service is still being tried");

        test_agent::set_elevated(true);
        start_in_process(&mut page);
        assert_eq!(test_agent::in_process_starts(), 2, "the button stays usable after a refusal");
        assert_eq!(in_process(h), InProcess::Running);
        assert!(!splash_shown(&page), "{:#?}", page.tree());
    }

    #[guinea::test(iterations = 4, exclusive = "agent")]
    fn the_service_still_wins_after_the_monitor_in_process_was_refused(h: &mut Harness) {
        start(h, false);
        let h = &*h;
        let mut page = mount(h);

        after(h, &mut page, 5);
        start_in_process(&mut page);
        assert_eq!(in_process(h), InProcess::NotElevated);

        test_agent::set_up(true);
        after(h, &mut page, 3);
        assert_eq!(agent(h), AgentConnectionState::Connected);
        assert!(!splash_shown(&page), "{:#?}", page.tree());
    }

    fn open_pane(h: &Harness, page: &mut Mounted<'_, ShellLayout>) {
        h.dispatch::<SidebarState>().emit(SetOpen(true));
        page.settle();
    }

    fn tile_shown(page: &Mounted<'_, ShellLayout>, chart: SidebarChart) -> bool {
        page.find(ui::SidebarMark::tile(chart)).is_some()
    }

    #[guinea::test(iterations = 4, exclusive = "agent")]
    fn a_hidden_chart_leaves_the_pane_and_comes_back(h: &mut Harness) {
        start(h, true);
        let h = &*h;
        let mut page = mount(h);
        open_pane(h, &mut page);
        for chart in SidebarChart::ALL {
            assert!(tile_shown(&page, chart), "{chart:?}: {:#?}", page.tree());
        }
        assert!(page.find(ui::SidebarMark::Charts).is_some(), "the menu that picks them");

        page.click(ui::SidebarMark::show(SidebarChart::Disk)).settle();
        page.settle();
        assert!(!tile_shown(&page, SidebarChart::Disk), "{:#?}", page.tree());
        assert!(tile_shown(&page, SidebarChart::Network));
        assert!(!h.state::<SettingsState>().sidebar_charts.shows(SidebarChart::Disk));

        page.click(ui::SidebarMark::show(SidebarChart::Disk)).settle();
        page.settle();
        assert!(tile_shown(&page, SidebarChart::Disk), "{:#?}", page.tree());
        assert!(h.state::<SettingsState>().sidebar_charts.shows(SidebarChart::Disk));
    }

    #[guinea::test(iterations = 4, exclusive = "agent")]
    fn rate_charts_name_the_top_of_their_scale(h: &mut Harness) {
        start(h, true);
        let h = &*h;
        let mut page = mount(h);
        open_pane(h, &mut page);
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
        open_pane(h, &mut page);
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
        open_pane(h, &mut page);

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
}
