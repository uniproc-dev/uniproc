use app_contracts::features::settings::{AppTheme, SettingsState};
use domain::features::settings::SettingsFeature;
use guinea::feature::FeatureInitContext;
use guinea::winui::{layout, Layout, LayoutCx, UpdateCx, UseRoute};
use windows_reactor::{ColorScheme, View, WindowBackdrop, WindowTheme, WindowVisuals};

use crate::route_memory;
use crate::routes::Route;

struct Window;

#[expect(non_upper_case_globals)]
impl Window {
    const Width: f64 = 1000.0;
    const Height: f64 = 700.0;
}

#[derive(Default)]
pub struct MainWindow {
    scheme: ColorScheme,
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

#[layout]
impl Layout for MainWindow {
    type Params = crate::routes::MainWindowParams;
    type Installs = SettingsFeature;
    type Message = ColorScheme;

    fn install(ctx: &FeatureInitContext, _params: &Self::Params) -> anyhow::Result<Self::Installs> {
        ctx.install(&())
    }

    fn init(ctx: &FeatureInitContext, _params: &Self::Params) -> Self {
        crate::xaml_resources::override_navigation_view_resources();
        crate::window_press::install();
        let window = Self {
            routes: route_memory::open(ctx),
            ..Self::default()
        };
        guicons::set_theme(icon_theme(window.scheme));
        window
    }

    fn update(&mut self, scheme: ColorScheme, _cx: &mut UpdateCx<'_, Self>) {
        self.scheme = scheme;
        guicons::set_theme(icon_theme(scheme));
    }

    fn view(&self, cx: &mut LayoutCx<'_, '_, Self>) -> View {
        let on_scheme = cx.on(|scheme: ColorScheme| scheme);
        cx.on_color_scheme(on_scheme);
        let (settings, _) = cx.read::<SettingsState>();
        cx.window_visuals(
            WindowVisuals::new()
                .client_size(Window::Width, Window::Height)
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

        windows_reactor::provide(ui::theme::scheme_context(), self.scheme, cx.outlet())
    }
}
