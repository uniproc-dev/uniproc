use app_contracts::features::agent_link::{AgentLinkState, StartInProcess};
use app_contracts::features::agents::AgentConnectionState;
use app_contracts::features::metrics::MetricsState;
use app_contracts::features::settings::{AppTheme, SettingsState, ShowSidebarChart};
use app_contracts::features::sidebar::{SetOpen, SetWidth, SidebarState};
use domain::features::agent_link::{AgentLinkDeps, AgentLinkFeature};
use domain::features::agents::providers::windows::AGENT_SERVICE_DISPLAY_NAME;
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

use crate::layouts::ProcessesArea;
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
        unreachable_service: (link.windows == AgentConnectionState::GaveUp)
            .then_some(AGENT_SERVICE_DISPLAY_NAME),
        on_start_in_process: Callback::new(move |()| dispatch.emit(StartInProcess)),
    }))
}

#[derive(Default)]
pub struct ShellLayout {
    scheme: ColorScheme,
    charts: [Chart; 5],
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

    fn init(_ctx: &FeatureInitContext, _params: &Self::Params) -> Self {
        crate::xaml_resources::override_navigation_view_resources();
        crate::window_press::install();
        let shell = Self::default();
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
                let (_, dispatch) = cx.state::<SidebarState, _>();
                dispatch.emit(SetWidth(width.round() as u64));
            }
            ShellMsg::OpenChanged(open) => {
                let (sidebar, dispatch) = cx.state::<SidebarState, _>();
                if sidebar.open != open {
                    dispatch.emit(SetOpen(open));
                }
            }
        }
    }

    fn view(&self, cx: &mut LayoutCx<'_, Self>) -> View {
        let on_scheme = cx.on(ShellMsg::Scheme);
        cx.on_color_scheme(on_scheme);
        let (settings, settings_dispatch) = cx.use_reducer::<SettingsState, _>();
        cx.window_visuals(
            WindowVisuals::new()
                .client_size(WINDOW_WIDTH, WINDOW_HEIGHT)
                .backdrop(WindowBackdrop::Mica)
                .theme(window_theme(settings.theme)),
        );

        let current = cx.use_route::<Route>();
        cx.use_effect("uniproc::remember_route", current.clone(), move || {
            route_memory::remember(&current);
            None
        });

        let (sidebar, _) = cx.use_reducer::<SidebarState, _>();
        let (metrics, _) = cx.use_reducer::<MetricsState, _>();
        let (link, link_dispatch) = cx.use_reducer::<AgentLinkState, _>();
        let l10n = ui::l10n::use_tr(cx);
        let nav = cx.use_navigate::<Route>();

        let selected_tag = if cx.child_is::<Services>() {
            "services"
        } else if cx.child_is::<Wsl>() {
            "wsl"
        } else if cx.child_is::<ProcessesArea>() {
            "processes"
        } else if cx.child_is::<Settings>() {
            "settings"
        } else {
            ""
        };

        let on_select = Callback::new(move |tag: Option<String>| match tag.as_deref() {
            Some("processes") => nav.to(Route::Processes {}),
            Some("services") => nav.to(Route::Services {}),
            Some("wsl") => nav.to(Route::Wsl {}),
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
            units: settings.byte_units,
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
    use std::time::Duration;

    use guinea::app::Harness;
    use guinea::winui::harness::{Mounted, Outlet};
    use guinea_plugin_l10n::L10nPlugin;
    use guinea_plugin_store::amethystate::store::builder::Backend;
    use guinea_plugin_store::StorePlugin;
    use uuid::Uuid;

    use app_contracts::features::agent_link::InProcess;
    use app_contracts::features::settings::SidebarChart;
    use app_contracts::features::agents::{
        AgentConnectionState, WindowsAction, WindowsActionRequest, WindowsActionResponse,
    };

    use super::*;
    use crate::test_agent;

    fn mount(h: &Harness) -> Mounted<'_, ShellLayout> {
        Mounted::<ShellLayout>::mount_at(&h.segment(), crate::routes::ShellLayoutParams::default(), Route::Processes {})
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

    fn start(h: &mut Harness, agent_up: bool) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        test_agent::reset(agent_up);
        h.plugin(StorePlugin::at(dir.path().join("settings")).backend(Backend::Json))
            .unwrap()
            .plugin(L10nPlugin::<app_contracts::l10n::L10n>::new("en"))
            .unwrap()
            .feature(test_agent::FakeAgentFeature)
            .unwrap()
            .provide(AgentLinkDeps {
                start_in_process: test_agent::start_in_process,
            });
        dir
    }

    fn after(h: &Harness, page: &mut Mounted<'_, ShellLayout>, seconds: u64) {
        h.advance(Duration::from_secs(seconds));
        page.settle();
    }

    #[guinea::test(iterations = 16, exclusive = "store")]
    fn the_splash_gives_up_after_five_attempts_and_keeps_trying(h: &mut Harness) {
        let _store = start(h, false);
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

    #[guinea::test(iterations = 4, exclusive = "store")]
    fn the_settings_item_opens_the_settings_page(h: &mut Harness) {
        let _store = start(h, true);
        let h = &*h;
        let mut page = mount(h);

        page.click_text("Settings").settle();
        page.settle();

        assert_eq!(page.navigated::<Route>(), [Route::Settings {}]);
    }

    fn unreachable_line(page: &Mounted<'_, ShellLayout>) -> Option<String> {
        page.tree()
            .find(ui::SplashMark::Unreachable)
            .and_then(|node| node.text.clone())
    }

    #[guinea::test(iterations = 8, exclusive = "store")]
    fn the_splash_names_the_service_it_cannot_reach(h: &mut Harness) {
        let _store = start(h, false);
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
            format!("Can’t connect to the “\u{2068}{AGENT_SERVICE_DISPLAY_NAME}\u{2069}” service")
        );

        after(h, &mut page, 3);
        assert!(unreachable_line(&page).is_some(), "still named while it keeps trying");

        test_agent::set_up(true);
        after(h, &mut page, 3);
        assert_eq!(unreachable_line(&page), None, "{:#?}", page.tree());
    }

    #[guinea::test(iterations = 4, exclusive = "store")]
    fn the_splash_is_gone_as_soon_as_the_agent_answers(h: &mut Harness) {
        let _store = start(h, true);
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

    #[guinea::test(iterations = 4, exclusive = "store")]
    fn the_splash_offers_the_monitor_in_process_after_five_seconds(h: &mut Harness) {
        let _store = start(h, false);
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

    #[guinea::test(iterations = 8, exclusive = "store")]
    fn the_monitor_in_process_takes_over_from_the_service(h: &mut Harness) {
        let _store = start(h, false);
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

        let kill = h.publish(WindowsActionRequest::new(Uuid::new_v4(), WindowsAction::Kill { pid: 42 }));
        kill.settle();
        assert!(kill.chain().published::<WindowsActionResponse>(), "{:#?}", kill.chain());
        assert!(
            matches!(test_agent::in_process_actions().as_slice(), [WindowsAction::Kill { pid: 42 }]),
            "{:?}",
            test_agent::in_process_actions()
        );
    }

    #[guinea::test(iterations = 8, exclusive = "store")]
    fn the_monitor_in_process_needs_an_elevated_uniproc(h: &mut Harness) {
        let _store = start(h, false);
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

    #[guinea::test(iterations = 4, exclusive = "store")]
    fn the_service_still_wins_after_the_monitor_in_process_was_refused(h: &mut Harness) {
        let _store = start(h, false);
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

    #[guinea::test(iterations = 4, exclusive = "store")]
    fn a_hidden_chart_leaves_the_pane_and_comes_back(h: &mut Harness) {
        let _store = start(h, true);
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

    #[guinea::test(iterations = 4, exclusive = "store")]
    fn rate_charts_name_the_top_of_their_scale(h: &mut Harness) {
        let _store = start(h, true);
        let h = &*h;
        let mut page = mount(h);
        open_pane(h, &mut page);
        after(h, &mut page, 1);

        let tree = page.tree();
        let scale = |chart: SidebarChart| {
            tree.find(ui::SidebarMark::tile(chart))
                .unwrap_or_else(|| panic!("{chart:?}: {tree:#?}"))
                .find_text("100%: \u{2068}100 KB/s\u{2069}")
                .is_some()
        };
        assert!(scale(SidebarChart::Disk), "{tree:#?}");
        assert!(scale(SidebarChart::Network), "{tree:#?}");
        assert!(!scale(SidebarChart::Cpu), "a percent chart needs no scale");
    }
}
