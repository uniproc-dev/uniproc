use app_contracts::features::metrics::MetricsState;
use app_contracts::features::processes::MachineSummary;
use app_contracts::features::settings::{ByteUnits, SidebarChart, SidebarCharts, UpdateInterval};
use guicons::icon;
use guinea::prelude::Load;
use guinea::winui::MarkExt;
use guinea_widgets::chart::Chart;
use guinea_widgets::resize::{resize_handle, RESIZE_HANDLE_WIDTH};
use windows_reactor::{
    AutoSuggestBox, Border, Button, ButtonStyle, Callback, CheckBox, ChildrenControl, ContentControl, Flyout,
    FlyoutExt, FlyoutPlacement, Grid, GridChildExt, GridLength, HorizontalAlignment, KeyedView, LayoutControl,
    NavigationView, NavigationViewBackButtonVisible, NavigationViewItem, NavigationViewPaneDisplayMode,
    StackPanel, Thickness, TitleBar, VerticalAlignment, View, WindowTitleBarHeight,
};

use crate::format;
use crate::l10n::L10n;
use crate::theme::{size, space, Palette};
use crate::widgets::metric_chart::{
    chart_level, chart_title, metric_chart, metric_mini_bar, MetricChart, Scale,
};
use crate::widgets::separator;
use crate::widgets::text::text;

struct Title;

#[expect(non_upper_case_globals)]
impl Title {
    const SearchWidth: f64 = 240.0;
}

struct Sidebar;

#[expect(non_upper_case_globals)]
impl Sidebar {
    const MinWidth: f64 = 200.0;
    const MaxWidth: f64 = 500.0;
    const MetricHeight: f64 = 48.0;
    const MenuIcon: f64 = 16.0;
}

pub struct ShellProps<'a> {
    pub l10n: &'a L10n,
    pub palette: Palette,
    pub open: bool,
    pub width: f64,
    pub selected_tag: &'a str,
    pub content: View,
    pub splash: Option<View>,
    pub metrics: &'a MetricsState,
    pub units: ByteUnits,
    pub cadence_ms: u64,
    pub charts: &'a [Chart; 5],
    pub shown: SidebarCharts,
    pub on_show_chart: Callback<(SidebarChart, bool)>,
    pub on_select: Callback<Option<String>>,
    pub on_resize: Callback<f64>,
    pub on_open_changed: Callback<bool>,
}

fn nav_items(l10n: &L10n) -> Vec<(&'static str, String, View)> {
    vec![
        (
            "processes",
            l10n.shell_nav_processes(),
            icon!(apps_list).size(size::NavIcon).build(),
        ),
        (
            "services",
            l10n.shell_nav_services(),
            icon!(puzzle).size(size::NavIcon).build(),
        ),
        (
            "wsl",
            l10n.shell_nav_wsl(),
            icon!(linux).size(size::NavIcon).build(),
        ),
        (
            "system",
            l10n.shell_nav_system(),
            icon!(system).size(size::NavIcon).build(),
        ),
    ]
}

fn footer_nav_items(l10n: &L10n) -> Vec<(&'static str, String, View)> {
    vec![(
        "settings",
        l10n.shell_nav_settings(),
        icon!(settings).size(size::NavIcon).build(),
    )]
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

fn cadence_ms(chart: SidebarChart, update_ms: u64) -> u64 {
    match chart {
        SidebarChart::Disk | SidebarChart::Network => UpdateInterval::machine(update_ms).as_millis() as u64,
        SidebarChart::Cpu | SidebarChart::Memory | SidebarChart::Gpu => update_ms,
    }
}

fn scale(chart: SidebarChart, units: ByteUnits) -> Scale {
    match chart {
        SidebarChart::Disk | SidebarChart::Network => Scale::Rate(units),
        SidebarChart::Cpu | SidebarChart::Memory | SidebarChart::Gpu => Scale::Percent,
    }
}

fn detail(chart: SidebarChart, machine: Option<&MachineSummary>, units: ByteUnits) -> Option<String> {
    let machine = machine?;
    match chart {
        SidebarChart::Cpu => Some(format!(
            "{:.1} / {:.1} GHz",
            machine.cpu_current_mhz as f64 / 1000.0,
            machine.cpu_max_mhz as f64 / 1000.0
        )),
        SidebarChart::Memory => Some(format::bytes(units, machine.memory_total_bytes)),
        SidebarChart::Gpu => Some(format::bytes(units, machine.gpu_memory_used_bytes)),
        SidebarChart::Disk | SidebarChart::Network => None,
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
            .content(text(chart_title(chart)))
    });
    Button::new()
        .mark(SidebarMark::Charts)
        .style(ButtonStyle::Subtle)
        .horizontal_alignment(HorizontalAlignment::Right)
        .content(icon!(more_horizontal).size(Sidebar::MenuIcon).build())
        .flyout_with(Flyout::rich(StackPanel::new().children(toggles)).placement(FlyoutPlacement::TopEdgeAlignedRight))
}

fn metrics_pane_footer(props: &ShellProps<'_>) -> View {
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
                chart: &props.charts[chart as usize],
                kind: chart,
                history: history(metrics, chart),
                scale: scale(chart, props.units),
                cadence_ms: cadence_ms(chart, props.cadence_ms),
                height: Sidebar::MetricHeight,
                detail: detail(chart, machine, props.units),
                palette,
            })),
        )
    }));

    Border::new()
        .padding(Thickness::xy(space::Compact, space::Control))
        .horizontal_alignment(HorizontalAlignment::Stretch)
        .content(StackPanel::new().spacing(space::Compact).children((charts_menu(props), tiles)))
}

pub fn shell_view(mut props: ShellProps<'_>) -> View {
    let title_bar = TitleBar::new()
        .preferred_height(WindowTitleBarHeight::Tall)
        .is_pane_toggle_button_visible(false)
        .grid_row(0);

    let (backdrop, title_bar, body, handle): (View, TitleBar, View, View) = match props.splash.take() {
        Some(splash) => (
            Border::new()
                .grid_row(0)
                .grid_row_span(2)
                .content(splash)
                .into(),
            title_bar,
            View::empty(),
            View::empty(),
        ),
        None => {
            let title_bar = title_bar.title(props.l10n.shell_window_title()).content(
                AutoSuggestBox::new()
                    .placeholder_text(props.l10n.shell_search())
                    .width(Title::SearchWidth)
                    .vertical_alignment(VerticalAlignment::Center),
            );
            let handle = sidebar_resize_handle(&props);
            (View::empty(), title_bar, navigation(props), handle)
        }
    };

    Grid::new()
        .rows([GridLength::Auto, GridLength::Star(1.0)])
        .columns([GridLength::Star(1.0)])
        .children((backdrop, title_bar, body, handle))
        .into()
}

fn navigation(props: ShellProps<'_>) -> View {
    let l10n = props.l10n;
    let selected_tag = props.selected_tag;
    let to_nav_item = |(tag, label, icon): (&'static str, String, View)| {
        (
            tag,
            NavigationViewItem::new()
                .tag(tag)
                .content(label)
                .icon(icon)
                .is_selected(tag == selected_tag),
        )
    };
    let nav_items: Vec<_> = nav_items(l10n).into_iter().map(to_nav_item).collect();
    let footer_nav_items: Vec<_> = footer_nav_items(l10n).into_iter().map(to_nav_item).collect();

    NavigationView::new()
        .menu_items(nav_items)
        .footer_menu_items(footer_nav_items)
        .pane_footer(metrics_pane_footer(&props))
        .on_selected_tag_changed(props.on_select.clone())
        .is_pane_open(props.open)
        .pane_display_mode(NavigationViewPaneDisplayMode::Left)
        .is_pane_toggle_button_visible(true)
        .on_is_pane_open_changed(props.on_open_changed.clone())
        .is_back_button_visible(NavigationViewBackButtonVisible::Collapsed)
        .is_settings_visible(false)
        .open_pane_length(props.width)
        .grid_row(1)
        .content(props.content)
        .into()
}

fn sidebar_resize_handle(props: &ShellProps<'_>) -> View {
    if props.open {
        Border::new()
            .margin(Thickness::new(
                props.width - RESIZE_HANDLE_WIDTH / 2.0,
                0.0,
                0.0,
                0.0,
            ))
            .horizontal_alignment(HorizontalAlignment::Left)
            .grid_row(1)
            .content(
                resize_handle(props.width, props.on_resize.clone())
                    .min(Sidebar::MinWidth)
                    .max(Sidebar::MaxWidth)
                    .build(),
            )
            .into()
    } else {
        View::empty()
    }
}
