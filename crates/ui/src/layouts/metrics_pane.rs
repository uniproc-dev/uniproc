use app_contracts::features::metrics::{History, MetricsState};
use app_contracts::features::processes::MachineSummary;
use app_contracts::features::settings::{SidebarChart, SidebarCharts, Units};
use guicons::icon;
use guinea::prelude::Load;
use guinea::winui::MarkExt;
use guinea_widgets::chart::Chart;
use windows_reactor::{
    Border, Button, ButtonStyle, Callback, CheckBox, Flyout, FlyoutExt, FlyoutPlacement,
    HorizontalAlignment, StackPanel, Thickness, View, keyed,
};

use crate::format::{self, Rate};
use crate::l10n::L10n;
use crate::theme::{Palette, size, space};
use crate::widgets::metric_chart::{
    MetricChart, Scale, chart_level, metric_chart, metric_mini_bar,
};
use crate::widgets::text::text;
use crate::widgets::{nothing, separator};

struct Pane;

#[expect(non_upper_case_globals)]
impl Pane {
    const MetricHeight: f64 = 48.0;
    const MenuIcon: f64 = 16.0;
}

pub struct SidebarChartsProps<'a> {
    pub l10n: &'a L10n,
    pub palette: Palette,
    pub open: bool,
    pub metrics: &'a MetricsState,
    pub units: Units,
    pub cadence_ms: u64,
    pub charts: &'a [Chart; 5],
    pub shown: SidebarCharts,
    pub on_show_chart: Callback<(SidebarChart, bool)>,
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

fn history(metrics: &MetricsState, chart: SidebarChart) -> &Load<History> {
    match chart {
        SidebarChart::Cpu => &metrics.cpu_history,
        SidebarChart::Memory => &metrics.memory_history,
        SidebarChart::Disk => &metrics.disk_history,
        SidebarChart::Network => &metrics.network_history,
        SidebarChart::Gpu => &metrics.gpu_history,
    }
}

fn scale(chart: SidebarChart, units: Units) -> Scale {
    match chart {
        SidebarChart::Disk => Scale::Rate(Rate::disk(units)),
        SidebarChart::Network => Scale::Rate(Rate::network(units)),
        SidebarChart::Cpu | SidebarChart::Memory | SidebarChart::Gpu => Scale::Percent,
    }
}

fn detail(chart: SidebarChart, machine: Option<&MachineSummary>, units: Units) -> Option<String> {
    let units = units.bytes;
    let machine = machine?;
    match chart {
        SidebarChart::Memory => Some(format::bytes(units, machine.memory_total_bytes)),
        SidebarChart::Gpu => Some(format::bytes(units, machine.gpu_memory_used_bytes)),
        SidebarChart::Cpu | SidebarChart::Disk | SidebarChart::Network => None,
    }
}

fn corner(l10n: &L10n, chart: SidebarChart, machine: Option<&MachineSummary>) -> Option<String> {
    match chart {
        SidebarChart::Cpu => {
            Some(l10n.metric_chart_cpu_frequency(format::ghz(machine?.cpu_current_mhz)))
        }
        SidebarChart::Memory | SidebarChart::Gpu | SidebarChart::Disk | SidebarChart::Network => {
            None
        }
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

fn charts_menu(props: &SidebarChartsProps<'_>) -> View {
    let toggles: [View; 5] = SidebarChart::ALL.map(|chart| {
        let on_show = props.on_show_chart.clone();
        CheckBox::new()
            .mark(SidebarMark::show(chart))
            .is_checked(props.shown.shows(chart))
            .on_is_checked_changed(move |shown: Option<bool>| {
                on_show.call((chart, shown == Some(true)));
            })
            .content(text(chart_toggle_label(props.l10n, chart)))
            .into()
    });
    Button::new()
        .mark(SidebarMark::Charts)
        .style(ButtonStyle::Subtle)
        .horizontal_alignment(HorizontalAlignment::Right)
        .content(icon!(more_horizontal).size(Pane::MenuIcon).build_element())
        .flyout_with(
            Flyout::rich(StackPanel::new().children(toggles))
                .placement(FlyoutPlacement::TopEdgeAlignedRight),
        )
}

pub fn sidebar_charts(props: &SidebarChartsProps<'_>) -> View {
    let metrics = props.metrics;
    let palette = props.palette;
    let shown: Vec<SidebarChart> = props.shown.shown().collect();

    if !props.open {
        if shown.is_empty() {
            return nothing();
        }
        let bars = shown.iter().map(|&chart| {
            let scale = scale(chart, props.units);
            let level = chart_level(history(metrics, chart), scale);
            keyed(
                chart.id(),
                metric_mini_bar(palette, chart, level, scale.warns()),
            )
        });
        let column = std::iter::once(keyed("above", separator(palette)))
            .chain(bars)
            .chain(std::iter::once(keyed("below", separator(palette))));
        return Border::new()
            .padding(Thickness::xy(size::NavIcon, space::Control))
            .horizontal_alignment(HorizontalAlignment::Center)
            .content(
                StackPanel::new()
                    .spacing(space::Control)
                    .keyed_children(column),
            )
            .into();
    }

    let machine = metrics.machine.ready();
    let tiles = shown.iter().map(|&chart| {
        keyed(
            chart.id(),
            Border::new()
                .mark(SidebarMark::tile(chart))
                .content(metric_chart(MetricChart {
                    l10n: props.l10n,
                    chart: &props.charts[chart as usize],
                    kind: chart,
                    history: history(metrics, chart),
                    scale: scale(chart, props.units),
                    cadence_ms: props.cadence_ms,
                    height: Pane::MetricHeight,
                    detail: detail(chart, machine, props.units),
                    corner: corner(props.l10n, chart, machine),
                    palette,
                })),
        )
    });
    let column = std::iter::once(keyed("menu", charts_menu(props))).chain(tiles);

    Border::new()
        .padding(Thickness::xy(space::Compact, space::Control))
        .horizontal_alignment(HorizontalAlignment::Stretch)
        .content(
            StackPanel::new()
                .spacing(space::Compact)
                .keyed_children(column),
        )
        .into()
}
