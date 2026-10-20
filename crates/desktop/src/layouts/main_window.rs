use app_contracts::features::settings::{AppTheme, SettingsState};
use domain::features::settings::SettingsFeature;
use guinea::feature::Install;
use guinea::winui::{layout, Layout, LayoutCx, UpdateCx};
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
    type Message = ColorScheme;

    fn install(ctx: &Install, store: amethystate::Store) -> anyhow::Result<Self> {
        ctx.install::<SettingsFeature>(&())?;
        crate::xaml_resources::override_navigation_view_resources();
        let window = Self {
            routes: Some(route_memory::open(&store)),
            ..Self::default()
        };
        guicons::set_theme(icon_theme(window.scheme));
        Ok(window)
    }

    fn update(&mut self, scheme: ColorScheme, _cx: &mut UpdateCx<'_, Self>) {
        self.scheme = scheme;
        guicons::set_theme(icon_theme(scheme));
    }

    fn view(&self, cx: &mut LayoutCx<'_, '_, Self>) -> View {
        let bus = cx.bus();
        cx.use_effect_guard("uniproc::window_press", (), move || {
            crate::window_press::install(move || bus.publish(app_contracts::features::window::PressedAway))
        });
        let drawn = self.scheme;
        let on_scheme = cx.on_some(move |scheme: ColorScheme| (scheme != drawn).then_some(scheme));
        cx.on_color_scheme(on_scheme);
        let (settings, _) = cx.read::<SettingsState>();
        cx.window_visuals(
            WindowVisuals::new()
                .client_size(Window::Width, Window::Height)
                .backdrop(WindowBackdrop::Mica)
                .theme(window_theme(settings.theme)),
        );

        let current = cx.navigate::<Route>().current();
        let routes = self.routes.clone();
        cx.use_effect("uniproc::remember_route", current.clone(), move || {
            if let (Some(routes), Some(current)) = (&routes, &current) {
                route_memory::remember(routes, current);
            }
            None
        });

        let shell = cx.mount::<crate::layouts::Shell>(&());
        windows_reactor::provide(ui::theme::scheme_context(), self.scheme, shell)
    }
}
