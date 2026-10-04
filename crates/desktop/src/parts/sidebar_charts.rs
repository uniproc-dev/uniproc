use app_contracts::features::metrics::MetricsState;
use app_contracts::features::settings::{SettingsState, ShowSidebarChart};
use app_contracts::features::sidebar::SidebarState;
use domain::features::metrics::MetricsFeature;
use guinea::feature::FeatureInitContext;
use guinea::winui::{page, Page, PageCx};
use guinea_widgets::chart::Chart;
use ui::theme::{scheme_context, Palette};
use windows_reactor::{Callback, View};

#[derive(Default)]
pub struct SidebarCharts {
    charts: [Chart; 5],
}

#[page]
impl Page for SidebarCharts {
    type Installs = MetricsFeature;

    fn install(ctx: &FeatureInitContext, _params: &Self::Params) -> anyhow::Result<Self::Installs> {
        ctx.install(&())
    }

    fn view(&self, cx: &mut PageCx<'_, '_, Self>) -> View {
        let (metrics, _) = cx.read::<MetricsState>();
        let (settings, dispatch) = cx.read::<SettingsState>();
        let (sidebar, _) = cx.read::<SidebarState>();
        let l10n = ui::l10n::use_tr(cx);
        let palette = Palette::of(cx.use_context(scheme_context()));
        ui::sidebar_charts(&ui::SidebarChartsProps {
            l10n: &l10n,
            palette,
            open: sidebar.open,
            metrics: &metrics,
            units: settings.units,
            cadence_ms: settings.update_interval_ms,
            charts: &self.charts,
            shown: settings.sidebar_charts,
            on_show_chart: Callback::new(move |(chart, shown)| dispatch.emit(ShowSidebarChart(chart, shown))),
        })
    }
}
