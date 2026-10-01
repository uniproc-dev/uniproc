use app_contracts::features::metrics::MetricsState;
use app_contracts::features::processes::MachineSummary;
use app_contracts::features::settings::{ByteUnits, SidebarChart};
use guicons::icon;
use guinea::prelude::Load;
use guinea::winui::MarkExt;
use windows_reactor::{
    Border, Button, ButtonStyle, CheckBox, ChildrenControl, ContentControl, Flyout, FlyoutExt, FlyoutPlacement,
    HorizontalAlignment, KeyedView, LayoutControl, StackPanel, Thickness, View,
};

use super::shell::ShellProps;
use crate::format;
use crate::l10n::L10n;
use crate::theme::{size, space};
use crate::widgets::metric_chart::{chart_level, metric_chart, metric_mini_bar, MetricChart, Scale};
use crate::widgets::separator;
use crate::widgets::text::text;

struct Pane;

#[expect(non_upper_case_globals)]
impl Pane {
    const MetricHeight: f64 = 48.0;
    const MenuIcon: f64 = 16.0;
}

#[derive(guinea::Mark, Clone, Copy, PartialEq, Eq, Debug)]
pub enum SidebarMark {
    Charts,
    ShowCpu,
    ShowMemory,
    ShowDisk,
    ShowNetwork,
    ShowGpu,
    Cpu,
    Memory,
    Disk,
    Network,
    Gpu,
}

impl SidebarMark {
    pub fn tile(chart: SidebarChart) -> Self {
        match chart {
            SidebarChart::Cpu => Self::Cpu,
            SidebarChart::Memory => Self::Memory,
            SidebarChart::Disk => Self::Disk,
            SidebarChart::Network => Self::Network,
            SidebarChart::Gpu => Self::Gpu,
        }
    }

    pub fn show(chart: SidebarChart) -> Self {
        match chart {
            SidebarChart::Cpu => Self::ShowCpu,
            SidebarChart::Memory => Self::ShowMemory,
            SidebarChart::Disk => Self::ShowDisk,
            SidebarChart::Network => Self::ShowNetwork,
            SidebarChart::Gpu => Self::ShowGpu,
        }
    }
}

fn history(metrics: &MetricsState, chart: SidebarChart) -> &Load<Vec<(u64, f32)>> {
    match chart {
        SidebarChart::Cpu => &metrics.cpu_history,
        SidebarChart::Memory => &metrics.memory_history,
        SidebarChart::Disk => &metrics.disk_history,
        SidebarChart::Network => &metrics.network_history,
        SidebarChart::Gpu => &metrics.gpu_history,
    }
}

fn scale(chart: SidebarChart, units: ByteUnits) -> Scale {
    match chart {
        SidebarChart::Disk | SidebarChart::Network => Scale::Rate(units),
        SidebarChart::Cpu | SidebarChart::Memory | SidebarChart::Gpu => Scale::Percent,
    }
}

fn detail(l10n: &L10n, chart: SidebarChart, machine: Option<&MachineSummary>, units: ByteUnits) -> Option<String> {
    let machine = machine?;
    match chart {
        SidebarChart::Cpu => Some(l10n.metric_chart_cpu_frequency(
            format::ghz(machine.cpu_current_mhz),
            format::ghz(machine.cpu_max_mhz),
        )),
        SidebarChart::Memory => Some(format::bytes(units, machine.memory_total_bytes)),
        SidebarChart::Gpu => Some(format::bytes(units, machine.gpu_memory_used_bytes)),
        SidebarChart::Disk | SidebarChart::Network => None,
    }
}

fn chart_toggle_label(l10n: &L10n, chart: SidebarChart) -> String {
    match chart {
        SidebarChart::Cpu => l10n.shell_chart_cpu(),
        SidebarChart::Memory => l10n.shell_chart_memory(),
        SidebarChart::Disk => l10n.shell_chart_disk(),
        SidebarChart::Network => l10n.shell_chart_network(),
        SidebarChart::Gpu => l10n.shell_chart_gpu(),
    }
}

fn charts_menu(props: &ShellProps<'_>) -> View {
    let toggles = SidebarChart::ALL.map(|chart| {
        let on_show = props.on_show_chart.clone();
        CheckBox::new()
            .mark(SidebarMark::show(chart))
            .is_checked(props.shown.shows(chart))
            .on_is_checked_changed(move |shown: bool| {
                let _ = on_show.call((chart, shown));
            })
            .content(text(chart_toggle_label(props.l10n, chart)))
    });
    Button::new()
        .mark(SidebarMark::Charts)
        .style(ButtonStyle::Subtle)
        .horizontal_alignment(HorizontalAlignment::Right)
        .content(icon!(more_horizontal).size(Pane::MenuIcon).build())
        .flyout_with(Flyout::rich(StackPanel::new().children(toggles)).placement(FlyoutPlacement::TopEdgeAlignedRight))
}

pub(super) fn metrics_pane(props: &ShellProps<'_>) -> View {
    let metrics = props.metrics;
    let palette = props.palette;
    let shown: Vec<SidebarChart> = props.shown.shown().collect();

    if !props.open {
        if shown.is_empty() {
            return View::empty();
        }
        let bars = View::keyed_fragment(shown.iter().map(|&chart| {
            let scale = scale(chart, props.units);
            let level = chart_level(history(metrics, chart), scale);
            KeyedView::new(chart.id(), metric_mini_bar(palette, chart, level, scale.warns()))
        }));
        return Border::new()
            .padding(Thickness::xy(size::NavIcon, space::Control))
            .horizontal_alignment(HorizontalAlignment::Center)
            .content(
                StackPanel::new()
                    .spacing(space::Control)
                    .children((separator(palette), bars, separator(palette))),
            );
    }

    let machine = metrics.machine.ready();
    let tiles = View::keyed_fragment(shown.iter().map(|&chart| {
        KeyedView::new(
            chart.id(),
            Border::new().mark(SidebarMark::tile(chart)).content(metric_chart(MetricChart {
                l10n: props.l10n,
                chart: &props.charts[chart as usize],
                kind: chart,
                history: history(metrics, chart),
                scale: scale(chart, props.units),
                cadence_ms: props.cadence_ms,
                height: Pane::MetricHeight,
                detail: detail(props.l10n, chart, machine, props.units),
                palette,
            })),
        )
    }));

    Border::new()
        .padding(Thickness::xy(space::Compact, space::Control))
        .horizontal_alignment(HorizontalAlignment::Stretch)
        .content(StackPanel::new().spacing(space::Compact).children((charts_menu(props), tiles)))
}
